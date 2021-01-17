use std::default::Default;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone)]
pub struct UrlNode {
    pub name: String,
    pub children: Vec<UrlNode>,
    pub data: Option<DynamicObject>
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
    pub outfile: String,
    pub domain: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkObject {
    pub domain: Option<String>,
    pub file_path: String,
    pub link_path: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentValue {
    pub key: String,
    pub value: String
}