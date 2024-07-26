use std::path::Path;
use colored::Colorize;
use yaml_rust2::{Yaml, YamlLoader};

///
/// A configuration data for server
///
pub struct ServerConfiguration {
    config_yaml: Vec<Yaml>,
}

impl ServerConfiguration {
    ///
    /// Loads configuration.
    ///
    /// returns: Option<Self>: Either configuration or None if failed to load
    ///
    pub fn load(path: &Path) -> Option<Self>{
        let data = std::fs::read_to_string(path);
        if data.is_err(){
            println!("{}:{}", "error".red().bold().underline(), " Can not read rc file".clear());
            return None;
        }
        let configuration_result = YamlLoader::load_from_str(&data.unwrap());
        if configuration_result.is_err(){
            println!("{}:{}", "error".red().bold().underline(), " Can not parse rc file".clear());
            return None;
        }
        Some(ServerConfiguration {
            config_yaml: configuration_result.unwrap()
        })
    }

    ///
    /// Gets a path to the storage
    ///
    /// returns: Option<&Path>: path to a storage directory
    ///
    pub fn get_storage_path(&self) -> Option<&Path>{
        let str_path = self.config_yaml[0]["storage_path"].as_str();
        if str_path.is_none(){
            return None;
        }
        Some(Path::new(str_path.unwrap()))
    }

    ///
    /// Gets a path to the modules directory
    ///
    /// returns: Option<&Path>: path to a storage directory
    ///
    pub fn get_modules_path(&self) -> Option<&Path>{
        let str_path = self.config_yaml[0]["modules_path"].as_str();
        if str_path.is_none(){
            return None;
        }
        Some(Path::new(str_path.unwrap()))
    }
    
    ///
    /// Gets a listener address
    /// 
    /// returns: Option<String>: a listener bind address
    ///
    pub fn get_listener_address(&self) -> Option<String>{
        todo!()
    }
}