//TODO: stop 2 config files on same layer, generate url tree

use std::io::Read;
use std::fs::OpenOptions;
use serde_json;
pub use structs::*;

mod structs;

//TODO: maybe make struct for tree
pub fn get_url_tree() -> () {
    let settings = read_settings();

    let root_path = Path::from_str(&settings.root);
    let all_config_files = read_config_files(&settings.config_files, &root_path);

    for config in all_config_files {
        println!("{}", config.path.original);
    }
}

fn read_settings() -> ServerSettings {
    let mut settings_file = OpenOptions::new()
        .read(true)
        .open("server_settings.json")
        .expect("Critical Error: could not open settings file");

    let mut settings_json = String::new();
    settings_file.read_to_string(&mut settings_json)
        .expect("Critical Error: could not read settings file");

    let settings: ServerSettings = serde_json::from_str(&settings_json)
        .expect(&format!("Critical Error: invalid settings file"));
    
    settings
}

fn read_config_files(config_filenames: &Vec<String>, parent_path: &Path) -> Vec<ConfigWithPath> {
    let mut config_list: Vec<ConfigWithPath> = Vec::new();
    
    for filename in config_filenames {
        let full_rel_filepath = format!("{}/{}", parent_path.original, filename);
        let self_config = read_config_file(&full_rel_filepath);
        let self_path = Path::from_str(&full_rel_filepath).parent().unwrap(); // All config files must be in directory

        let mut child_config_list = read_config_files(&self_config.config_files, &self_path);
        
        config_list.append(&mut child_config_list);
        config_list.push(ConfigWithPath {
            path: self_path,
            config: self_config
        });
    }

    config_list
}

fn read_config_file(filename: &str) -> Config {
    let mut config_file = OpenOptions::new()
        .read(true)
        .open(filename)
        .expect(&format!("Critical Error: could not open config file \"{}\"", filename));

    let mut config_json = String::new();
    config_file.read_to_string(&mut config_json)
        .expect(&format!("Critical Error: could not read config file \"{}\"", filename));
    
    serde_json::from_str(&config_json)
        .expect(&format!("Critical Error: invalid config file \"{}\"", filename))
}