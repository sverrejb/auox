use crate::models::TokenData;
use serde::Deserialize;
use std::{fs, path::PathBuf};

fn app_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("auox"))
}

fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|base| base.join("auox"))
}

fn config_file() -> Option<PathBuf> {
    let dir = match app_config_dir() {
        Some(path) => path,

        None => {
            panic!("Could not determine config directory")
        }
    };

    println!("App config dir: {}", dir.display());

    // Create the directory if needed
    std::fs::create_dir_all(&dir).expect("Failed to create config dir");

    // Then use it for your files
    let config_path = dir.join("config.toml");
    println!("Config file path: {}", config_path.display());
    Some(config_path)
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub client_id: String,
}

pub fn get_config() -> AppConfig {
    if let Some(conf_path) = config_file() {
        let file = fs::read(conf_path)
            .expect("could not read config.toml")
            .iter()
            .map(|c| *c as char)
            .collect::<String>();

        let appconfig: AppConfig =
            toml::from_str(&file).expect("config.toml is not in proper format");

        appconfig
    } else {
        panic!("Unable to load config. Panicing....");
    }
}

pub fn read_access_token() -> String {
    let dir = match app_data_dir() {
        Some(path) => path,
        None => {
            panic!("Could not determine data directory")
        }
    };

    println!("App data dir: {}", dir.display());

    // Create the directory if needed
    std::fs::create_dir_all(&dir).expect("Failed to create data dir");

    let token_path = dir.join("auth.json");

    let file_content = fs::read_to_string(&token_path).expect("Could not read token.json");

    let token_data: TokenData =
        serde_json::from_str(&file_content).expect("token.json is not in proper format");

    println!("Token data: {}", token_data.access_token);
    token_data.access_token
}
