use std::net::{ TcpListener, TcpStream };
use std::sync::{ Arc, Mutex, MutexGuard };
use std::fs::{ self, File };
use std::io::{ Read, Write };
use std::thread;
use std::collections::HashMap;
use std::process::Command;
use std::fmt::Display;
use std::time::{ Instant, Duration };
use std::env;
use std::error::Error;
use native_tls::{ Identity, TlsAcceptor, TlsStream };
use rand;
use crate::{ log, Result, ServerError };
use crate::url_tree::{ UrlTree, UrlNode, Path, FileType, DynamicObject, FileData };
use crate::protocol::{ self, Request, Response, StatusCode };

const BUFFER_SIZE: usize = 2048;
const TEMP_DIR: &str = crate::TEMP_DIR;
const FILE_MAP_DEL_TIME: u64 = 300; // How often the file id removal thread should be run (seconds)

lazy_static! {
    static ref UNIQUE_FILE_LIST: Mutex<HashMap<u64, Instant>> = Mutex::new(HashMap::new());
}

pub fn run_server(tree: UrlTree) {
    let tree = Arc::new(tree);

    // Get certificate
    let cert_src = &tree.settings.tls_profile;
    let cert_passwd = &tree.settings.profile_password;
    let mut cert_file = File::open(cert_src).expect("Critical Error: Failed to open certificate");
    let mut certificate: Vec<u8> = vec![];
    cert_file.read_to_end(&mut certificate).expect("Critical Error: Failed to read certifcate");
    let identity = Identity::from_pkcs12(&certificate, cert_passwd).expect("Critical Error: Failed to create identity (bad certificate)");

    // Create Tcp Listeners based on ipv4/6 settings
    let mut listeners: Vec<TcpListener> = Vec::new();
    if tree.settings.ipv6 {
        let listener = TcpListener::bind("[::]:1965").expect("Critical Error: Failed to bind to address (ipv6)");
        listeners.push(listener);
    }
    if tree.settings.ipv4 {
        let listener = TcpListener::bind("0.0.0.0:1965").expect("Critical Error: Failed to bind to address (ipv4)");
        listeners.push(listener);
    }

    // Create Tls wrapper for acceptors based on certificate
    let acceptor = TlsAcceptor::new(identity.clone()).expect("Critical Error: Failed to initialize acceptor");
    let acceptor = Arc::new(acceptor);
    
    // Spawn thread for removing unused file ids
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(FILE_MAP_DEL_TIME));
            clear_unique_file_list().and_then(|_| Ok(())).unwrap(); // Stupid stuff to silence warning
        }
    });

    // Stop if neither ipv6 or ipv4 is enabled
    if listeners.len() == 0 {
        panic!("Critical Error: Either ipv4 or ipv6 must be enabled in the server settings to run the program");
    }

    // Log start of server
    log("Info: Started Server");

    // Start server thread(s)
    if listeners.len() == 1 {
        let listener0 = listeners.pop().unwrap();
        handle_server(listener0, acceptor, tree.clone());
    }
    else {
        let acceptor_copy = acceptor.clone();
        let tree_copy = tree.clone();
        let listener0 = listeners.pop().unwrap();
        let listener1 = listeners.pop().unwrap();

        thread::spawn(move || handle_server(listener0, acceptor_copy, tree_copy));
        handle_server(listener1, acceptor.clone(), tree.clone())
    }
}

fn handle_server(listener: TcpListener, acceptor: Arc<TlsAcceptor>, tree: Arc<UrlTree>)
{
    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                let acceptor = acceptor.clone();
                let tree = tree.clone();

                thread::spawn(move || {
                    let client = match acceptor.accept(client) {
                        Ok(val) => val,
                        Err(_) => return
                    };
                    handle_client(client, tree);
                });
            },
            Err(_) => continue
        }
    }
}

fn handle_client(mut client: TlsStream<TcpStream>, tree: Arc<UrlTree>) {
    let mut buffer = [0; BUFFER_SIZE];

    // Read and parse request from client
    let num_bytes = match client.read(&mut buffer) {
        Ok(val) => val,
        Err(_) => {
            shutdown_client(client);
            return;
        }
    };
    let request = match protocol::parse_request(&buffer[0..num_bytes]) {
        Ok(val) => val,
        Err(_) => {
            shutdown_client(client);
            return;
        }
    };

    // Generate response and send it to client
    let response = handle_request(&request, &tree);
    match client.write(&response) {
        Ok(_) => (),
        Err(_) => ()
    };

    shutdown_client(client);
}

fn shutdown_client(mut client: TlsStream<TcpStream>) {
    match client.shutdown() {
        Ok(_) => (),
        Err(_) => ()
    }
}

fn handle_request(request: &Request, tree: &UrlTree) -> Vec<u8> {
    let node = match search_in_tree(tree, &request.domain, &request.path) {
        Ok(val) => val,
        Err(err) => return get_err_response(err, tree.settings.serve_errors, tree.settings.log)
    };
    let (body, mime) = match get_resource(node, &request.query) {
        Ok(val) => val,
        Err(err) => return get_err_response(err, tree.settings.serve_errors, tree.settings.log)
    };

    // Create meta field
    let meta;
    if mime.starts_with("text") {
        if mime == "text/gemini" && tree.settings.default_lang.is_some() { 
            meta = format!(
                "text/gemini; charset={}; lang={}",
                &tree.settings.default_charset,
                &tree.settings.default_lang.as_ref().unwrap()
            );
        }
        else {
            meta = format!("{}; {}", mime, &tree.settings.default_charset);
        }
    }
    else {
        meta = mime.to_string();
    }


    Response::new(StatusCode::Success, meta, body).build()
}

fn search_in_tree<'a>(tree: &'a UrlTree, domain: &str, path: &str) -> Result<&'a UrlNode> {
    let not_found_err = Err(ServerError::new(
        format!(
            "Error: Resource not found. Path: {}",
            path
        ),
        StatusCode::NotFound
    ));
    
    for root in &tree.roots {
        // Find root node with the correct domain
        if root.name == domain {
            // Get the requested path
            let node = match root.get_child_from_path(&Path::from_str(path)) {
                Some(val) => val,
                None => return not_found_err
            };

            return Ok(node)
        }
    }

    not_found_err
}

fn get_err_response(err: ServerError, serve_errors: bool, log: bool) -> Vec<u8> {
    let ServerError { message, status_code, is_meta } = err;

    if log && !is_meta {
        let err_msg = message.clone();
        thread::spawn(move || crate::log(&err_msg)); // Logging could be time consuming
    }

    let response;
    if serve_errors || is_meta {
        response = Response {
            status_code: status_code,
            meta: message,
            body: Vec::new()
        }
    }
    else {
        response = Response {
            status_code: status_code,
            meta: String::new(),
            body: Vec::new()
        }
    }

    response.build()
}

// Returns binary data and mime-type
fn get_resource<'a>(node: &'a UrlNode, query: &Option<String>) -> Result<(Vec<u8>, &'a str)> {
    let not_found_err = || Err(ServerError::new(
        String::from("Error: Resource not found"),
        StatusCode::NotFound
    ));

    let result = match &node.data {
        Some( // Case data is already loaded
            FileData {
                meta_data,
                binary_data: Some(binary_data)
            }
        ) => {
            let binary_data = binary_data.clone();
            let mime_type = meta_data.get_mime_type();

            Ok((
                binary_data,
                mime_type
            ))
        },
        Some( // Case data has not been loaded / Dynamic
            FileData {
                meta_data,
                binary_data: None
            }
        ) => {
            let binary_data = load_data(meta_data, query)?;
            let mime_type = meta_data.get_mime_type();

            Ok((
                binary_data,
                mime_type
            ))
        }
        // Case node does not exist (file not found)
        None => not_found_err()
    };

    result
}

fn load_data(file_type: &FileType, query: &Option<String>) -> Result<Vec<u8>> {
    let internal_error = |err: &dyn Display| Err(ServerError::new(
        format!("Error: Resource could not be retrieved. {}", err),
        StatusCode::TemporaryFailure
    ));
    
    if let FileType::Normal(val) = file_type { 
        match fs::read(&val.path.original) {
            Ok(val) => return Ok(val),
            Err(err) => return internal_error(&err)
        }
    }
    else if let FileType::Link(val) = file_type {
        match fs::read(&val.file_path) {
            Ok(val) => return Ok(val),
            Err(err) => return internal_error(&err)
        }
    }
    else if let FileType::Dynamic(val) = file_type {
        return load_dynamic_content(val, query);
    }
    
    internal_error(&"")
}

fn load_dynamic_content(dynamic_object: &DynamicObject, query: &Option<String>) -> Result<Vec<u8>> {
    let cgi_error = |err: &dyn Display| Err(ServerError::new(
        format!("Error: Process failed to generate content. {}", err),
        StatusCode::CGIError
    ));

    // Insert an entry for the file
    let temp_file_num;
    let mut file_map = get_unique_file_list()?;
    loop {
        let random_num = rand::random::<u64>();
        if file_map.contains_key(&random_num) {
            continue;
        }

        file_map.insert(random_num, Instant::now());
        temp_file_num = random_num;
        break;
    } 

    // Get the path
    let temp_file_path = match env::current_dir() {
        Ok(mut val) => {
            val.push(TEMP_DIR);
            val.push(temp_file_num.to_string());
            val
        },
        Err(err) => return cgi_error(&err)
    };
    let temp_file_path = temp_file_path.display().to_string();

    // Create process
    let mut process = Command::new(&dynamic_object.program_path);
    process.current_dir(&dynamic_object.cmd_working_dir);
    process.envs(
        dynamic_object.cmd_env
        .iter()
        .map(|val| (val.key.clone(), val.value.clone()))
    );

    // Add command line arguments
    if dynamic_object.args.len() != 0 {
        process.args(dynamic_object.args.clone());
    }

    // Add path name
    process.arg(
        format!(
            "unique_file_path=\"{}\"",
            temp_file_path
        )
    );

    // Handle query
    if let Some(query_options) = &dynamic_object.query {
        if let Some(query_value) = query {
            process.arg(
                format!(
                    "query=\"{}\"",
                    query_value
                )
            );
        }
        else {
            let status_code = match query_options.private {
                true => StatusCode::SensitiveInput,
                false => StatusCode::Input
            };

            return Err(ServerError {
                message: query_options.display_text.clone(),
                is_meta: true,
                status_code: status_code
            });
        }
    }

    // Start process
    let mut process = match process.spawn() {
        Ok(val) => val,
        Err(err) => return cgi_error(&err)
    };

    // Poll process for completion, exit if time over
    let start_time = Instant::now();
    let gen_time = dynamic_object.gen_time.unwrap(); // gen_time is always set at this point
    while start_time.elapsed().as_secs() < gen_time {
        let poll_exit = process.try_wait();
        if let Ok(Some(_)) = poll_exit {
            return read_and_remove(&temp_file_path, temp_file_num);
        }
        else {
            continue;
        }
    }

    cgi_error(&"")
}

fn read_and_remove(file_name: &str, unique_num: u64) -> Result<Vec<u8>> {
    let cgi_error = |err: &dyn Display| Err(ServerError::new(
        format!("Error: Failed to read generated content. {}", err),
        StatusCode::CGIError
    ));

    if !std::path::Path::new(file_name).exists() { // This entry will later be removed automatically
        return cgi_error(&"No content was generated");
    }

    // Get the data
    let data = match fs::read(file_name) {
        Ok(val) => val,
        Err(err) => return cgi_error(&err)
    };

    // Remove the entry
    let mut file_map = match get_unique_file_list() {
        Ok(val) => val,
        Err(_) => return Ok(data)
    };
    file_map.remove(&unique_num);

    Ok(data)
}

fn get_unique_file_list() -> Result<MutexGuard<'static, HashMap<u64, Instant>>> {
    let cgi_error = || Err(ServerError::new(
        String::from("Error: Too many clients at once"),
        StatusCode::CGIError
    ));

    for _ in 0..10 {
        match UNIQUE_FILE_LIST.lock() {
            Ok(val) => return Ok(val),
            Err(_) => ()
        };

        let sleep_time = Duration::from_millis((rand::random::<f32>() * 25.0) as u64); // Random time to avoid conflicts
        thread::sleep(sleep_time)
    }

    cgi_error()
}

fn clear_unique_file_list() -> std::result::Result<(), Box<dyn Error>> {
    let mut file_map = get_unique_file_list()?;
    let file_ids: Vec<u64> = file_map.iter().map(|val| *val.0).collect();

    for file_id in file_ids {
        let file_name = format!("{}/{}", TEMP_DIR, file_id);

        // If file does not exist just remove entry
        if !std::path::Path::new(&file_name).exists() {
            file_map.remove(&file_id);
            continue;
        }

        match fs::remove_file(&file_name) {
            Ok(_) => (),
            Err(_) => continue
        }

        // Check again if file exists before removing, b/c even with no error, file is not immeadiately deleted
        if !std::path::Path::new(&file_name).exists() {
            file_map.remove(&file_id);
        }
    }

    Ok(())
}