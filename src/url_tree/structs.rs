use std::default::Default;
use std::fmt::{ Display, Formatter };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone)]
pub enum FileType {
    Dynamic(DynamicObject),
    Link(LinkObject),
    Normal((Path, String))
}

#[derive(Debug, Clone)]
pub struct UrlNode {
    pub name: String,
    pub domain: String,
    pub children: Vec<UrlNode>,
    pub data: Option<FileType>,
    pub file: bool
}

impl UrlNode {
    // This will not do anything if the file is already present
    pub fn add_file_path(&mut self, path: &Path, file_data: FileType) {
        let new_node = UrlNode {
            name: path.last(),
            domain: self.domain.clone(),
            children: Vec::new(),
            data: Some(file_data),
            file: true
        };

        if let None = path.parent() {
            if !self.has_child(&path.last()) {
                self.children.push(new_node);
            }
        }
        else if let None = self.get_child_from_path(path) {
            let parent_path = path.parent().unwrap();
            self.add_dir_path(&parent_path);
            let path_end = self.get_child_from_path(&parent_path).unwrap(); // Just added path, must be found
            path_end.children.push(new_node);
        }
    }

    // This will not do anything if the file is already present
    pub fn add_dir_path(&mut self, path: &Path) {
        if path.components.len() == 0 {
            return;
        }

        let domain = self.domain.clone();
        let mut node_ref = self;
        let path_components = path.components.clone();

        let mut i = 0;
        let len = path_components.len();
        while i < len {
            let name = &path_components[i];
            
            // If child exists, move reference
            if node_ref.has_child(name)
            {
                let child = node_ref.get_child(name).unwrap();
                node_ref = child;
                i += 1;
                continue;
            }

            // Else add new child node
            let new_node = UrlNode {
                domain: domain.clone(),
                name: name.clone(),
                children: Vec::new(),
                data: None,
                file: false
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
            node_ref = match node_ref.get_child(&components[i]) {
                Some(val) => val,
                None => return
            };

            i += 1;
        }

        let mut i = 0;
        let len = node_ref.children.len();
        while i < len {
            if node_ref.children[i].name == path.last() {
                node_ref.children.remove(i);
                break;
            }

            i += 1;
        }
    }

    pub fn get_child_from_path<'a>(&'a mut self, path: &Path) -> Option<&'a mut UrlNode> {
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

    fn has_child(&self, child_name: &str) -> bool {
        for child in &self.children {
            if child.name == child_name {
                return true;
            }
        }

        false
    }

    fn get_child<'a>(&'a mut self, child_name: &str) -> Option<&'a mut UrlNode> {
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
                format_string += &format!("{}\t--> {}", &tabs, child.name);
            }

            if i != len - 1 {
                format_string += "\n";
            }
            i += 1;
        }

        format_string
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

    pub fn from_parent(relative: &Self, parent: &Self) -> Self {
        let new_original = format!("{}/{}", relative.original, parent.original);
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    pub domain: String,
    pub root: String,
    pub tls_profile: String,
    pub error_profile: Option<String>,
    pub config_files: Vec<String>,
    pub max_dynamic_gen_time: u64,
    pub never_exit: bool,
    pub default_lang: String,
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
            domain: String::from(""),
            root: String::from("root"),
            tls_profile: String::from("profile.pfx"),
            error_profile: None,
            config_files: vec![
                String::from("root/config")
            ],
            max_dynamic_gen_time: 30,
            never_exit: false,
            default_lang: String::from("en"),
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
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub default_whitelist: bool,
    pub dynamic: Vec<DynamicObject>,
    pub link: Vec<LinkObject>,
    #[serde(default = "Vec::new")]
    pub config_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicObject {
    pub path: String,
    pub command: String,
    pub cmd_working_dir: String,
    pub cmd_env: Vec<EnvironmentValue>,
    pub pass_vals: bool,
    pub pass_temp_filename: bool,
    pub mime_type: String,
    pub path_name: String,
    pub domain: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkObject {
    pub domain: Option<String>,
    pub file_path: String,
    pub link_path: String,
    pub mime_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValue {
    pub key: String,
    pub value: String
}