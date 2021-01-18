//TODO: add dynamic and link to tree, handle domain, generation of links page (later)

use std::io::Read;
use std::fs::{ self, OpenOptions };
use serde_json;
pub use structs::*;

mod structs;

//TODO: maybe make struct for tree
pub fn get_url_tree() -> () {
    // Read top level settings
    let settings = read_settings();

    // Read all lower level config
    let root_path = Path::from_str(&settings.root);
    let mut all_config = read_all_config_files(&settings.config_files, &root_path);

    // If one level has two config files, either show warning or panic
    let len = all_config.len();
    let mut i = 0;
    while i < len {
        let mut j = 0;
        while j < len {
            if all_config[i].path.parent().unwrap().components == all_config[j].path.parent().unwrap().components && i != j { // Parents must exist for files
                if !settings.never_exit {
                    panic!("Error: There are two config files on the level \"{}\"", all_config[i].path.original);
                }
                else {
                    println!("Warning: There are two config files on the level \"{}\". Only the first one will be used", all_config[i].path.original);
                    all_config.remove(j);
                    j -= 1;
                }
            }
            j += 1;
        }
        i += 1;
    }

    // Sort config files from lowest depth to highest
    let mut sorted_config_list: Vec<ConfigWithPath> = Vec::with_capacity(all_config.len());
    loop {
        let mut min_depth = all_config[0].path.depth();
        let mut min_index = 0;
        
        let mut i = 0;
        while i < all_config.len() {
            let depth = all_config[i].path.depth();
            if depth < min_depth {
                min_depth = depth;
                min_index = i;
            }

            i += 1;
        }

        let min_config = all_config[min_index].clone();
        sorted_config_list.push(min_config);
        all_config.remove(min_index);

        if all_config.len() == 0 {
            break;
        }
    }

    // Create nodes
    let mut root_node = get_root_node(&settings);
    create_tree(&sorted_config_list, &mut root_node, settings.never_exit);
    println!("{}", root_node);
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

fn read_all_config_files(config_filenames: &Vec<String>, parent_path: &Path) -> Vec<ConfigWithPath> {
    let mut config_list: Vec<ConfigWithPath> = Vec::new();
    
    for filename in config_filenames {
        let full_rel_filepath = format!("{}/{}", parent_path.original, filename);
        let self_config = read_config_file(&full_rel_filepath);
        let self_path = Path::from_str(&full_rel_filepath);
        let self_parent_path = self_path.parent().unwrap(); // All config files must be in directory

        let mut child_config_list = read_all_config_files(&self_config.config_files, &self_parent_path);
        
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

fn get_root_node(settings: &ServerSettings) -> UrlNode {
    UrlNode {
        domain: settings.domain.clone(),
        name: settings.root.clone(),
        children: Vec::new(),
        data: None,
        file: false
    }
}

fn create_tree(config_list: &Vec<ConfigWithPath>, root_node: &mut UrlNode, never_exit: bool) {
    for config in config_list {
        // Paths
        let real_config_dir_path = config.path.parent().unwrap(); // All config files have a parent folder
        let config_dir_path = real_config_dir_path.skip_components(1);

        // Handle whitelist / blacklist:
        let all_files = find_all_files(&real_config_dir_path.original, never_exit);
            let all_file_paths: Vec<Path> = all_files.into_iter().map(|file_path| {
                Path::from_str(&file_path).skip_components(1)
            }).collect();

        if config.config.default_whitelist {
            // Add all files
            for file_path in all_file_paths {
                root_node.add_file_path(&file_path, FileType::Normal(file_path.clone()));
            }

            // Remove current blacklisted files
            for rel_path in &config.config.blacklist {
                let file_path;
                if config_dir_path.components.len() == 0 { // If file is directly under root
                    file_path = String::from(rel_path);
                }
                else {
                    file_path = format!("{}/{}", config_dir_path.original.clone(), rel_path);
                }
                let file_path = Path::from_str(&file_path);
                
                root_node.remove_path(&file_path);
            }
        }
        else { // Remove all sub-files otherwise
            for file_path in all_file_paths {
                root_node.remove_path(&file_path);
            }

            for rel_path in &config.config.whitelist {
                let file_path;
                if config_dir_path.components.len() == 0 { // If file is directly under root
                    file_path = String::from(rel_path);
                }
                else {
                    file_path = format!("{}/{}", config_dir_path.original.clone(), rel_path);
                }
                let file_path = Path::from_str(&file_path);
                
                root_node.add_file_path(&file_path, FileType::Normal(file_path.clone()));
            }
        }
    }
}

fn find_all_files(dir_path: &str, never_exit: bool) -> Vec<String> {
    let mut all_files: Vec<String> = Vec::new();

    // Get all sub-entries
    let read_entries = match fs::read_dir(dir_path) {
        Ok(val) => val,
        Err(_) => {
            if never_exit {
                println!("Warning: the directory {} could not be read", dir_path);
                return all_files;
            }
            else {
                panic!("Error: the directory {} could not be read", dir_path);
            }
        } 
    };

    for entry in read_entries {
        let entry = match entry {
            Ok(val) => val,
            Err(_) => {
                if never_exit {
                    println!("Warning: the directory {} could not be read", dir_path);
                    return all_files;
                }
                else {
                    panic!("Error: the directory {} could not be read", dir_path);
                }
            }
        };

        let entry_is_dir = match entry.file_type() {
            Ok(val) => val.is_dir(),
            Err(_) => {
                if never_exit {
                    println!("Warning: the directory {} could not be read", dir_path);
                    return all_files;
                }
                else {
                    panic!("Error: the directory {} could not be read", dir_path);
                }
            }
        };

        let path = entry.path();
        let path = path.to_str().unwrap(); // This can panic if the path is not utf-8
        if entry_is_dir {
            let mut sub_files = find_all_files(path, never_exit);
            all_files.append(&mut sub_files);
        }
        else {
            all_files.push(path.to_string());
        }
    }

    all_files.into_iter().map(|path| {
        path.replace("\\", "/")
    }).collect()
}