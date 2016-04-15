//! Handles config files and default config.
//!
//! Looks for `~/.ascii-invoicer.yml` and patches unset fields from `DEFAULT_CONFIG`
//!

#![warn(missing_docs,
        missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
//#![warn(missing_debug_implementations)]


use std::path::{Path,PathBuf};
use std::env::home_dir;
use util::yaml;
use util::yaml::{Yaml, YamlError};

const DEFAULT_LOCATION: &'static str = ".asciii.yml";

/// Looks for a configuration yaml in your HOME_DIR
pub struct ConfigReader{
    /// Path of config file
    pub path: PathBuf,
    defaults: Yaml,
    custom: Yaml,
    local: Yaml
}

impl ConfigReader{

    /// The Path of the config file.
    pub fn path_home() -> PathBuf {
        let home = home_dir().expect("Can't find HOME DIRECTORY");
        home.join(DEFAULT_LOCATION)
    }

    /// DEBUG: Opens config from `self.path()` and parses Yaml right away.
    #[cfg(feature = "debug")]
    pub fn new () -> Result<ConfigReader, YamlError> {
        let path = ConfigReader::path_home();
        let config = Ok(ConfigReader{
            path: path.to_owned(),
            defaults: try!(yaml::parse(&DEFAULT_CONFIG)),
            custom: yaml::open(&path).unwrap_or(Yaml::Null),
            local:  yaml::open(Path::new(&DEFAULT_LOCATION)).unwrap_or(Yaml::Null)
        });
        println!("loading config from {default_path:?} or {local_path:?}", default_path = path, local_path = DEFAULT_LOCATION);

        config
    }

    /// Opens config from `self.path()` and parses Yaml right away.
    #[cfg(not(feature = "debug"))]
    pub fn new () -> Result<ConfigReader, YamlError> {
        let path = ConfigReader::path_home();
        Ok(ConfigReader{
            path: path.to_owned(),
            defaults: try!(yaml::parse(&DEFAULT_CONFIG)),
            custom: yaml::open(&path).unwrap_or(Yaml::Null),
            local:  yaml::open(Path::new(&DEFAULT_LOCATION)).unwrap_or(Yaml::Null)
        })
    }


    /// Returns whatever it finds in that position
    pub fn get(&self, key:&str) -> Option<&Yaml>{
        yaml::get(&self.local, &key)
            .or_else(||yaml::get(&self.custom, &key))
            .or_else(||yaml::get(&self.defaults, &key))
    }

    /// Returns whatever it finds in that position
    ///
    /// Supports simple path syntax: `top/middle/child/node`
    pub fn get_path(&self, path:&str) -> Option<&Yaml>{
        yaml::get(&self.local, &path)
            .or_else(||yaml::get(&self.custom, &path))
            .or_else(||yaml::get(&self.defaults, &path))
    }

    /// Returns the string in the position or an empty string
    ///
    /// # Panics
    /// This panics if nothing is found.
    /// You should have a default config for everything that you use.
    pub fn get_str(&self, key:&str) -> &str {
        yaml::get_str(&self.local, &key)
            .or_else(||yaml::get_str(&self.custom, &key))
            .or_else(||yaml::get_str(&self.defaults, &key))
            .expect(&format!("Config file {} in field {} does not contain a string value", DEFAULT_LOCATION, key))
    }

    /// Returns the boolean in the position or `false`
    ///
    /// # Panics
    /// This panics if nothing is found.
    /// You should have a default config for everything that you use.
    pub fn get_bool(&self, key:&str) -> bool {
        self.get_path(key)
            .and_then(|y|y.as_bool())
            .expect(&format!("Config file {} in field {} does not contain a boolean value", DEFAULT_LOCATION, key))
    }

}

/// Default configuration that will be used if a value is not set in yaml file at DEFAULT_LOCATION
pub const DEFAULT_CONFIG: &'static str = include_str!("./default_config.yml");

#[test]
fn simple_reading(){
    assert!(ConfigReader::path_home().exists());
    let config = ConfigReader::new().unwrap();

    assert_eq!(config.get("manager_name").unwrap().as_str().unwrap(),
               config.get_str("manager_name"));

    assert_eq!(config.get("list/colors").unwrap().as_bool().unwrap(),
               config.get_bool("list/colors"));

    assert!(config.get_path(&"dirs").is_some());
    assert!(config.get_path(&"dirs/storage").is_some());
    assert!(config.get_path(&"dirs/working").is_some());
    assert!(config.get_path(&"dirs/storage").is_some());

}
