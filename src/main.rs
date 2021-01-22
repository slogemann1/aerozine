//TODO: add some unit tests, add text tree generation, add general documentation

extern crate native_tls;
extern crate serde;
extern crate serde_json;

use std::fmt::{ Display, Formatter };
use std::io;
use protocol::StatusCode;

mod server;
mod url_tree;
mod protocol;

fn main() {
    let tree = url_tree::get_url_tree();
    server::run_server(tree);
}

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
pub struct ServerError {
    pub message: String,
    pub status_code: StatusCode
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ServerError {
    pub fn from_str(msg: &str, status: StatusCode) -> Self {
        ServerError {
            message: String::from(msg),
            status_code: status
        }
    }

    pub fn new(msg: String, status: StatusCode) -> Self {
        ServerError {
            message: msg,
            status_code: status
        }
    }
}