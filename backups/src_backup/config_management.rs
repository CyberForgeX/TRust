// File: ./src/config_management.rs

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    encryption_enabled: bool,
    cache_size: usize,
    encryption_key_path: PathBuf,
}

impl Config {
    fn load(config_file: &str) -> Result<Config, io::Error> {
        let config_path = PathBuf::from(config_file);
        if config_path.exists() {
            let mut file = File::open(config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let config: Config = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Log that default configuration is used
            println!("Configuration file not found. Using default settings.");
            // Provide default configuration
            Ok(Config {
                encryption_enabled: true,
                cache_size: 1024,
                encryption_key_path: PathBuf::from("encryption_key.bin"),
            })
        }
    }

    fn save(&self, config_file: &str) -> Result<(), io::Error> {
        let config_path = PathBuf::from(config_file);
        let contents = serde_json::to_string_pretty(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}
