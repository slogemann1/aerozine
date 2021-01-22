//TODO: make sure random files for dynamic content are deleted or added to list to not be reused
//TODO: add links, dynamic objects, reading files if data not present

use std::net::{ TcpListener, TcpStream };
use std::sync::Arc;
use std::fs::File;
use std::io::{ Read, Write };
use std::thread;
use native_tls::{ Identity, TlsAcceptor, TlsStream };
use crate::{ Result, ServerError };
use crate::url_tree::{
    UrlTree, UrlNode, Path, FileType, NormalFile, LinkObject, DynamicObject, FileData
};
use crate::protocol::{ self, Request, Response, StatusCode };

const BUFFER_SIZE: usize = 2048;

pub fn run_server(tree: UrlTree) {
    let tree = Arc::new(tree);

    let cert_src = &tree.settings.tls_profile;
    let cert_passwd = &tree.settings.profile_password;
    let mut cert_file = File::open(cert_src).expect("Critical Error: Failed to open certificate");
    let mut certificate: Vec<u8> = vec![];
    cert_file.read_to_end(&mut certificate).expect("Critical Error: Failed to read certifcate");
    let identity = Identity::from_pkcs12(&certificate, cert_passwd).expect("Critical Error: Failed to create identity (bad certificate)");

    let mut listeners: Vec<TcpListener> = Vec::new();
    if tree.settings.ipv6 {
        let listener = TcpListener::bind("[::]:1965").expect("Critical Error: Failed to bind to address (ipv6)");
        listeners.push(listener);
    }
    if tree.settings.ipv4 {
        let listener = TcpListener::bind("0.0.0.0:1965").expect("Critical Error: Failed to bind to address (ipv4)");
        listeners.push(listener);
    }

    let acceptor = TlsAcceptor::new(identity.clone()).expect("Critical Error: Failed to initialize acceptor");
    let acceptor = Arc::new(acceptor);
    
    if listeners.len() == 0 {
        panic!("Critical Error: Either ipv4 or ipv6 must be enabled in the server settings to run the program");
    }
    else if listeners.len() == 1 {
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
        Err(err) => return get_err_response(err, tree.settings.serve_errors)
    };
    let (body, mime) = match get_resource(node) {
        Ok(val) => val,
        Err(err) => return get_err_response(err, tree.settings.serve_errors)
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

fn get_err_response(err: ServerError, serve_errors: bool) -> Vec<u8> {
    let ServerError { message, status_code } = err;

    let response;
    if serve_errors {
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
fn get_resource<'a>(node: &'a UrlNode) -> Result<(Vec<u8>, &'a str)> {
    let not_found_err = Err(ServerError::new(
        format!("Error: Resource not found"),
        StatusCode::NotFound
    ));

    let result = match &node.data {
        Some(
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
        _ => not_found_err //TODO: change to none
    };

    result
}

//TODO
fn log(message: &str) {
    println!("{}", message);
}