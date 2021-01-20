//TODO: generation of links page (later), other pages from settings

use std::io::Read;
use std::fs::{ self, OpenOptions };
use std::collections::HashMap;
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
    create_tree(&sorted_config_list, &mut root_node, &settings);

    // Remove config files
    for config in &sorted_config_list {
        let root_depth = Path::from_str(&settings.root).depth();
        let rel_path = config.path.skip_components(root_depth); // Get path with respect to root
        
        root_node.remove_path(&rel_path);
    }

    // Seperate domains
    let mut nodes_with_path: HashMap<String, Vec<(Path, UrlNode)>> = HashMap::new();
    let all_roots = seperate_roots(&root_node, Path::root(), &mut nodes_with_path);

    // Organize nodes again
    let mut organized_trees: Vec<UrlNode> = Vec::new();
    let domains_with_nodes = nodes_with_path.drain();
    for (domain, nodes_list) in domains_with_nodes {
        let mut root_node = UrlNode {
            name: domain.clone(),
            children: Vec::new(),
            data: None
        };

        for (path, node) in &nodes_list {
            root_node.add_file_path(&path, node.data.clone().unwrap());
        }

        organized_trees.push(root_node);
    }

    println!("Trees:\n\n");
    for tree in organized_trees {
        println!("{}\n\n", tree);
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
        name: settings.root.clone(),
        children: Vec::new(),
        data: None,
    }
}

fn create_tree(config_list: &Vec<ConfigWithPath>, root_node: &mut UrlNode, settings: &ServerSettings) {
    let never_exit = settings.never_exit;
    let root_dir = settings.root.clone();
    let root_path = Path::from_str(&root_dir);
    let root_depth = root_path.depth(); // For amount of values to skip

    for config in config_list {
        // Paths
        let real_config_dir_path = config.path.parent().unwrap(); // All config files have a parent folder
        let config_dir_path = real_config_dir_path.skip_components(root_depth);
        
        // Domain 
        let domain = match &config.config.domain {
            Some(val) => String::from(val),
            None => String::from(&settings.domain)
        };

        // Handle whitelist / blacklist:

        // Get all files with respect to root
        let all_files = find_all_files(&real_config_dir_path.original, never_exit);
        let all_file_paths: Vec<Path> = all_files.into_iter().map(|file_path| {
            Path::from_str(&file_path).skip_components(root_depth)
        }).collect();

        // Remove all sub-files
        for file_path in &all_file_paths {
            root_node.remove_path(file_path);
        }

        if config.config.default_whitelist {
            for file_path in all_file_paths {
                // Get path including root
                let path = Path::from_parent(&root_path, &file_path);

                // Add file
                let file_data = NormalFile {
                    domain: domain.clone(),
                    path: path.clone(),
                    mime_type: get_mime_type(&path)
                };
                root_node.add_file_path(
                    &file_path,
                    FileType::Normal(file_data)
                );
            }

            // Remove current blacklisted files
            for rel_path in &config.config.blacklist {
                // Get file path with respect to root
                let file_path;
                if config_dir_path.is_root() {
                    file_path = String::from(rel_path);
                }
                else {
                    file_path = format!("{}/{}", config_dir_path.original.clone(), rel_path);
                }
                let file_path = Path::from_str(&file_path);

                root_node.remove_path(&file_path);
            }
        }
        else {
            // Add all whitelisted files
            for rel_path in &config.config.whitelist {
                // Get file path with respect to root
                let file_path;
                if config_dir_path.is_root() {
                    file_path = Path::from_str(&rel_path);
                }
                else {
                    file_path = Path::from_parent(&config_dir_path, &Path::from_str(&rel_path));
                }
                
                // Get path including root 
                let path = Path::from_parent(&root_path, &file_path);

                // Add file
                let file_data = NormalFile {
                    domain: domain.clone(),
                    path: path.clone(),
                    mime_type: get_mime_type(&path)
                };
                root_node.add_file_path(
                    &file_path,
                    FileType::Normal(file_data)
                );
            }
        }

        // Handle links:
        for link_obj in &config.config.link {
            let mut link_obj = link_obj.clone();
            let rel_path = link_obj.link_path.clone(); // Relative link path

            // Set domain to config domain if not defined
            if let None = link_obj.domain {
                link_obj.domain = Some(domain.clone());
            }
            // Infer mime type if not defined
            if let None = link_obj.mime_type {
                link_obj.mime_type = Some(
                    get_mime_type(&Path::from_str(&rel_path))
                );
            }

            // Get link path with respect to root
            let (link_path, file_path);
            if config_dir_path.is_root() {
                link_path = Path::from_str(&rel_path);
                file_path = Path::from_parent(&root_path, &Path::from_str(&link_obj.file_path));
            }
            else {
                link_path = Path::from_parent(&config_dir_path, &Path::from_str(&rel_path));
                file_path = Path::from_parent(
                    &Path::from_parent(&root_path, &config_dir_path),
                    &Path::from_str(&link_obj.file_path)
                );
            }

            // Add file
            link_obj.file_path = file_path.original;
            root_node.add_file_path(
                &link_path,
                FileType::Link(link_obj)
            );
        }

        // Handle dynamic content:
        for dynamic_obj in &config.config.dynamic {
            let mut dynamic_obj = dynamic_obj.clone();

            // Infer mime type if not defined
            if let None = dynamic_obj.mime_type {
                dynamic_obj.mime_type = Some(
                    get_mime_type(&Path::from_str(&dynamic_obj.link_path))
                );
            }
            // Use default gen time if not defined
            if let None = dynamic_obj.gen_time {
                dynamic_obj.gen_time = Some(settings.max_dynamic_gen_time);
            }
            // Use config domain if not defined
            if let None = dynamic_obj.domain {
                dynamic_obj.domain = Some(domain.clone());
            }

            // Get link path relative to root
            let link_path;
            if config_dir_path.is_root() {
                link_path = Path::from_str(&dynamic_obj.link_path);
            }
            else {
                link_path = Path::from_parent(&config_dir_path, &Path::from_str(&dynamic_obj.link_path));
            }

            // Add path
            root_node.add_file_path(
                &link_path,
                FileType::Dynamic(dynamic_obj)
            );
        }
    }
}

fn seperate_roots(node: &UrlNode, path: Path, nodes_with_path: &mut HashMap<String, Vec<(Path, UrlNode)>>) {
    for child in &node.children {
        // Get path relative to root
        let rel_path;
        if path.is_root() {
            rel_path = Path::from_str(&child.name);
        }
        else {
            rel_path = Path::from_parent(&path, &Path::from_str(&child.name));
        }

        // Add file endpoints to hashmap recursively
        if child.children.len() != 0 {
            seperate_roots(child, rel_path.clone(), nodes_with_path);
        }
        match &child.data {
            Some(_) => {
                let domain = child.get_domain().to_string();
                let node_copy = child.clone();

                let node_list = nodes_with_path.entry(domain).or_insert(Vec::new());
                node_list.push((rel_path, node_copy));
            },
            None => continue
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

//TODO: add more mime types
fn get_mime_type(path: &Path) -> String {
    let file_name = path.last();
    let name_parts: Vec<&str> = file_name.split(".").collect(); // Path must at least contain 1 element
    let ext = name_parts[name_parts.len() - 1];

    let mime = match ext {
        "gmi" => "text/gemini",
        "txt" => "text/plain",
        "html" => "text/html",
        _ => "text/plain"
    };

    String::from(mime)
}