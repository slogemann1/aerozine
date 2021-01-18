extern crate native_tls;
extern crate serde;
extern crate serde_json;

use std::fmt::{ Display, Formatter };

mod server;
mod url_tree;

fn main() {
    url_tree::get_url_tree();
    println!("done");
    server::run_server();
}

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
struct ServerError {
    message: String
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}