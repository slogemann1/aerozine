use std::default::Default;
use std::fmt::{ Display, Formatter };
use std::fs;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone)]
pub struct UrlNode {
    pub name: String,
    pub children: Vec<UrlNode>,
    pub data: Option<FileData>,
}

#[derive(Debug, Clone)]
pub struct UrlTree {
    pub settings: ServerSettings,
    pub roots: Vec<UrlNode>,
}

#[derive(Debug, Clone)]
pub struct FileData {
    pub meta_data: FileType,
    pub binary_data: Option<Vec<u8>>
}

impl FileData {
    pub fn from_file_type(file_type: FileType, never_exit: bool) -> Self {
        let file_path = match &file_type {
            FileType::Dynamic(val) => return FileData {
                meta_data: FileType::Dynamic(val.clone()),
                binary_data: None
            },
            FileType::Link(val) => &val.file_path,
            FileType::Normal(val) => &val.path.original
        };

        let binary_data = match fs::read(file_path) {
            Ok(val) => val,
            Err(err) => {
                if never_exit {
                    println!("Warning: Could not read the file at {} to memory. {}", file_path, err);
                    return FileData {
                        meta_data: file_type,
                        binary_data: None
                    }
                }
                else {
                    panic!("Error: Could not read the file at {} to memory. {}", file_path, err);
                }
            }
        };

        FileData {
            meta_data: file_type,
            binary_data: Some(binary_data)
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    Dynamic(DynamicObject),
    Link(LinkObject),
    Normal(NormalFile)
}

impl FileType {
    pub fn get_mime_type<'a>(&'a self) -> &'a str {
        match self {
            FileType::Dynamic(val) => &val.mime_type.as_ref().unwrap(), // Mime-type has been initialized at this point
            FileType::Link(val) => &val.mime_type.as_ref().unwrap(), //Same as above
            FileType::Normal(val) => &val.mime_type
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalFile {
    pub domain: String,
    pub path: Path,
    pub mime_type: String
}

impl UrlNode {
    // This will not do anything if the file is already present
    pub fn add_file_path(&mut self, path: &Path, file_data: FileData) {
        let new_node = UrlNode {
            name: path.last(),
            children: Vec::new(),
            data: Some(file_data),
        };

        if let None = path.parent() {
            if !self.has_child(&path.last()) {
                self.children.push(new_node);
            }
            else {
                let child = self.get_child_mut(&path.last()).unwrap(); // Must have child due to previous check
                if child.get_domain() != new_node.get_domain() { // If domains differ add anyway
                    self.children.push(new_node);
                }
                else { // Else mutate value 
                    child.data = new_node.data;
                }
            }
        }
        else if let None = self.get_child_from_path_mut(path) {
            let parent_path = path.parent().unwrap();
            self.add_dir_path(&parent_path);
            let path_end = self.get_child_from_path_mut(&parent_path).unwrap(); // Just added path, must be found
            path_end.children.push(new_node);
        }
        else {
            let child = self.get_child_from_path_mut(path).unwrap(); // Must have child due to previous check
            if child.get_domain() != new_node.get_domain() { //If domains differ add anyway
                let parent_path = path.parent().unwrap();
                let path_end = self.get_child_from_path_mut(&parent_path).unwrap(); //Past must exist (previous check)
                path_end.children.push(new_node);
            }
            else { // Else mutate value
                child.data = new_node.data;
            }
        }
    }

    // This will not do anything if the file is already present
    pub fn add_dir_path(&mut self, path: &Path) {
        if path.components.len() == 0 {
            return;
        }

        let mut node_ref = self;
        let path_components = path.components.clone();

        let mut i = 0;
        let len = path_components.len();
        while i < len {
            let name = &path_components[i];
            
            // If child exists, move reference
            if node_ref.has_child(name)
            {
                let child = node_ref.get_child_mut(name).unwrap();
                node_ref = child;
                i += 1;
                continue;
            }

            // Else add new child node
            let new_node = UrlNode {
                name: name.clone(),
                children: Vec::new(),
                data: None,
            };

            let old_len = node_ref.children.len();
            node_ref.children.push(new_node);
            node_ref = &mut node_ref.children[old_len];

            i += 1;
        }
    }

    pub fn remove_path(&mut self, path: &Path) {
        let mut node_ref = self;
        let components = &path.components;

        let mut i = 0;
        let len = components.len();
        while i < len - 1 {
            node_ref = match node_ref.get_child_mut(&components[i]) {
                Some(val) => val,
                None => return
            };

            i += 1;
        }

        let mut i: i32 = 0;
        while (i as usize) < node_ref.children.len() {
            if node_ref.children[i as usize].name == path.last() {
                node_ref.children.remove(i as usize);
                i -= 1; // Remove all instances, no break
            }

            i += 1;
        }
    }

    pub fn get_child_from_path_mut<'a>(&'a mut self, path: &Path) -> Option<&'a mut UrlNode> {
        let mut node_ref = self;
        let components = &path.components;

        for name in components {
            node_ref = match node_ref.get_child_mut(&name) {
                Some(val) => val,
                None => return None
            };
        }

       Some(node_ref)
    }

    pub fn get_child_from_path<'a>(&'a self, path: &Path) -> Option<&'a UrlNode> {
        let mut node_ref = self;
        let components = &path.components;

        for name in components {
            node_ref = match node_ref.get_child(&name) {
                Some(val) => val,
                None => return None
            };
        }

       Some(node_ref)
    }

    // Only call this if the type is file and the data has been initialized
    pub fn get_domain<'a>(&'a self) -> &'a str {
        match &self.data.as_ref().unwrap().meta_data {
            FileType::Normal(val) => &val.domain,
            FileType::Link(val) => val.domain.as_ref().unwrap(),
            FileType::Dynamic(val) => val.domain.as_ref().unwrap()
        }
    }

    fn has_child(&self, child_name: &str) -> bool {
        for child in &self.children {
            if child.name == child_name {
                return true;
            }
        }

        false
    }

    fn get_child_mut<'a>(&'a mut self, child_name: &str) -> Option<&'a mut UrlNode> {
        let len = self.children.len();
        let mut i = 0;
        while i < len {
            if self.children[i].name == child_name {
                return Some(&mut self.children[i]);
            }
            i += 1;
        }

        None
    }

    fn get_child<'a>(&'a self, child_name: &str) -> Option<&'a UrlNode> {
        let len = self.children.len();
        let mut i = 0;
        while i < len {
            if self.children[i].name == child_name {
                return Some(&self.children[i]);
            }
            i += 1;
        }

        None
    }

    fn text_tree(&self, depth: usize) -> String {
        let arrow = {
            if depth == 0 {
                ""
            }
            else {
                "--> "
            }
        };
        let tabs: String = vec!['\t'; depth].into_iter().collect();
        let mut format_string = format!("{}{}{}\n", tabs, arrow, self.name);

        let mut i = 0;
        let len = self.children.len();
        for child in &self.children {
            if child.children.len() != 0 {
                let child_strings = child.text_tree(depth + 1);
                format_string += &format!("{}", child_strings);
            }
            else {
                format_string += &format!("{}\t--> {} ({})", &tabs, child.name, child.text_url());
            }

            if i != len - 1 {
                format_string += "\n";
            }
            i += 1;
        }

        format_string
    }

    fn text_url(&self) -> String {
        if let None = self.data {
            return String::new();
        }

        let text_path = match &self.data.as_ref().unwrap().meta_data {
            FileType::Normal(val) => &val.path.original,
            FileType::Link(val) => &val.file_path,
            FileType::Dynamic(_) => "\"dynamic\""
        };

        String::from(text_path)
    }
}

impl Display for UrlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text_tree(0))
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    pub original: String,
    pub components: Vec<String>
}

#[derive(Debug, Clone)]
pub struct ConfigWithPath {
    pub path: Path,
    pub config: Config
}

impl Path {
    pub fn from_str(path_str: &str) -> Self {
        let all_forward = path_str.replace("\\", "/");
        let components: Vec<String> = all_forward
            .split("/")
            .map(|val| String::from(val))
            .collect();

        Path {
            original: String::from(path_str),
            components
        }
    }

    pub fn from_parent(parent: &Self, relative: &Self) -> Self {
        let new_original = format!("{}/{}", parent.original, relative.original);
        Self::from_str(&new_original)
    }

    fn from_components(components: Vec<String>) -> Self {
        let original = components.join("/");

        Path {
            original,
            components
        }
    }

    pub fn parent(&self) -> Option<Self> {
        let len = self.components.len();
        if len <= 1 {
            None
        }
        else {
            let components = self.components.clone()[0..len-1].to_vec();

            Some(Self::from_components(components))
        }
    }

    pub fn depth(&self) -> usize {
        self.components.len()
    }

    pub fn skip_components(&self, n: usize) -> Self {
        let new_components = self.components[n..self.components.len()].to_vec();
        Self::from_components(new_components)
    }

    // This requires a non-empty Path
    pub fn last(&self) -> String {
        self.components[self.components.len() - 1].clone()
    }

    pub fn is_root(&self) -> bool {
        self.components.len() == 0
    }

    pub fn root() -> Self {
        Path::from_components(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub domain: String,
    pub root: String,
    pub tls_profile: String,
    pub profile_password: String,
    pub error_profile: Option<String>,
    pub config_files: Vec<String>,
    pub max_dynamic_gen_time: u64,
    pub never_exit: bool,
    pub serve_errors: bool,
    pub default_lang: Option<String>,
    pub default_charset: String,
    pub homepage: Option<String>,
    pub gen_doc_page: bool,
    pub doc_page_path: String,
    pub ipv4: bool,
    pub ipv6: bool
}

impl Default for ServerSettings {
    fn default() -> Self {
        ServerSettings {
            domain: String::from("localhost"),
            root: String::from("root"),
            tls_profile: String::from("profile.pfx"),
            profile_password: String::from("password"),
            error_profile: None,
            config_files: vec![
                String::from("config.json")
            ],
            max_dynamic_gen_time: 10,
            never_exit: false,
            serve_errors: false,
            default_lang: None,
            default_charset: String::from("utf-8"),
            homepage: None,
            gen_doc_page: true,
            doc_page_path: String::from("links.gmi"),
            ipv4: true,
            ipv6: false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub domain: Option<String>,
    #[serde(default = "Vec::new")]
    pub whitelist: Vec<String>,
    #[serde(default = "Vec::new")]
    pub blacklist: Vec<String>,
    pub default_whitelist: bool,
    #[serde(default = "Vec::new")]
    pub dynamic: Vec<DynamicObject>,
    #[serde(default = "Vec::new")]
    pub link: Vec<LinkObject>,
    #[serde(default = "Vec::new")]
    pub config_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicObject {
    pub link_path: String, // Relative
    pub command: String,
    pub cmd_working_dir: String, // Absolute
    pub cmd_env: Vec<EnvironmentValue>,
    #[serde(default = "Vec::new")]
    pub parameters: Vec<QueryParameter>,
    pub mime_type: Option<String>,
    pub gen_time: Option<u64>,
    pub domain: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkObject {
    pub domain: Option<String>,
    pub file_path: String,
    pub link_path: String,
    pub mime_type: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValue {
    pub key: String,
    pub value: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    pub parameter: String,
    pub private: bool
}