//TODO: add some unit tests, add text tree generation, add general documentation

extern crate native_tls;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate chrono;
#[macro_use]
extern crate lazy_static;

use std::fmt::{ Display, Formatter };
use std::error::Error;
use std::fs::{ self, OpenOptions };
use std::io::Write;
use chrono::offset::Local;

use protocol::StatusCode;

mod server;
mod url_tree;
mod protocol;

const TEMP_DIR: &str = "temp";
const LOG_FILE: &str = "log.txt";

fn main() {
    let tree = url_tree::get_url_tree();
    reset_temp(tree.settings.never_exit);
    server::run_server(tree);
}

pub type Result<T> = std::result::Result<T, ServerError>;

#[derive(Debug)]
pub struct ServerError {
    pub message: String, // Error Message
    pub is_meta: bool, // If the message is a meta value 
    pub status_code: StatusCode // Corresponding status code
}

impl ServerError {
    pub fn from_str(msg: &str, status: StatusCode) -> Self {
        ServerError {
            message: String::from(msg),
            status_code: status,
            is_meta: false
        }
    }

    pub fn new(msg: String, status: StatusCode) -> Self {
        ServerError {
            message: msg,
            status_code: status,
            is_meta: false
        }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ServerError {}

fn reset_temp(never_exit: bool) {
    let temp_path = std::path::Path::new(TEMP_DIR);

    if temp_path.exists() && temp_path.is_dir() {
        match fs::remove_dir_all(temp_path) {
            Ok(_) => (),
            Err(err) => {
                if never_exit {
                    log(&format!("Warning: The temp directory could not be removed. {}", err));
                }
                else {
                    panic!("Error: The temp directory could not be removed. {}", err)
                }
            }
        }
    }

    match fs::create_dir(temp_path) {
        Ok(_) => (),
        Err(err) => {
            if never_exit {
                log(&format!("Warning: The temp directory could not be created. {}", err));
            }
            else {
                panic!("Error: The temp directory could not be created. {}", err)
            }
        }
    }
}

//TODO
fn log(message: &str) {
    let time = Local::now();
    let time_formatted = time.format("%Y.%m.%d %H:%M:%S");
    let log_entry = format!("{} | {}\n", time_formatted, message);

    let mut log_file = match OpenOptions::new().create(true).append(true).open(LOG_FILE) {
        Ok(val) => val,
        Err(err) => {
            println!("Error: Failed to log entry. {}\nLog Message:\n{}", err, log_entry);
            return;
        }
    };

    match log_file.write_all(log_entry.as_bytes()) {
        Ok(_) => (),
        Err(err) => println!("Error: Failed to log entry. {}\nLog Message:\n{}", err, log_entry)
    };
}