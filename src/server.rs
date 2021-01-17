use std::net::{ TcpListener, TcpStream };
use std::sync::Arc;
use std::fs::File;
use std::str;
use std::io::{ Read, Write };
use std::thread;
use native_tls::{ Identity, TlsAcceptor, TlsStream };
use crate::Result;

const BUFFER_SIZE: usize = 2048;

pub fn run_server() {
    let cert_src = "./data/certificate.pfx";
    let mut cert_file = File::open(cert_src).expect("Failed to open certificate");
    let mut certificate: Vec<u8> = vec![];
    cert_file.read_to_end(&mut certificate).expect("Failed to read certifcate");
    let identity = Identity::from_pkcs12(&certificate, "password").expect("Failed to create identity (bad certificate)");

    let listener = TcpListener::bind("0.0.0.0:1965").expect("Failed to bind to address (ipv4)");
    let acceptor = TlsAcceptor::new(identity).expect("Failed to initialize acceptor");
    let acceptor = Arc::new(acceptor);


    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                let acceptor = acceptor.clone();
                thread::spawn(move || {
                    let client = match acceptor.accept(client) {
                        Ok(val) => val,
                        Err(_) => return
                    };
                    handle_client(client);
                });
            },
            Err(_) => continue
        }
    }
}

fn handle_client(mut client: TlsStream<TcpStream>) {
    let mut buffer = [0; BUFFER_SIZE];

    let num_bytes = match client.read(&mut buffer) {
        Ok(val) => val,
        Err(_) => {
            shutdown_client(client);
            return;
        }
    };
    let request = match str::from_utf8(&buffer[0..num_bytes]) {
        Ok(val) => val,
        Err(_) => {
            shutdown_client(client);
            return;
        }
    };

    let (header, body) = match handle_request(request) {
        Ok(val) => val,
        Err(err) => {
            log(&format!("Failed to handle request with the following error: \"{}\"", err));
            shutdown_client(client);
            return;
        }
    };
    match client.write(&header) {
        Ok(_) => (),
        Err(_) => ()
    };
    match client.write(&body) {
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

fn handle_request(request: &str) -> Result<(Box<&[u8]>, Box<&[u8]>)> {
    let header = "20 text/gemini\r\n";
    let header = Box::new(header.as_bytes());

    let body = "\
    # Hello World!\n\
    Documentation:\n\
    => gemini://gemini.circumlunar.space/docs/specification.gmi Docs!
    ";
    let body = Box::new(body.as_bytes());

    Ok((header, body))
}

//TODO
fn log(message: &str) {
    println!("{}", message);
}