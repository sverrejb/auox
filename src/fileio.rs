use crate::models::TokenData;
use log::debug;
use serde::Deserialize;
use std::{fs, path::PathBuf};

fn app_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("auox"))
}

fn app_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|base| base.join("auox"))
}

fn config_file_path() -> Option<PathBuf> {
    let dir = match app_config_dir() {
        Some(path) => path,

        None => {
            panic!("Could not determine config directory")
        }
    };

    debug!("App config dir: {}", dir.display());

    // Create the directory if needed
    std::fs::create_dir_all(&dir).expect("Failed to create config dir");

    // Then use it for your files
    let config_path = dir.join("config.toml");
    debug!("Config file path: {}", config_path.display());
    Some(config_path)
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: String,
    pub financial_institution: String,
}

pub fn get_config_file() -> AppConfig {
    if let Some(conf_path) = config_file_path() {
        if !conf_path.exists() {
            create_config_template(&conf_path);
        }

        let file = fs::read_to_string(&conf_path).expect("could not read config.toml");

        let appconfig: AppConfig =
            toml::from_str(&file).expect("config.toml is not in proper format");
        appconfig
    } else {
        panic!("Unable to determine config directory location");
    }
}

pub fn read_access_token_file() -> Option<TokenData> {
    let dir = match app_data_dir() {
        Some(path) => path,
        None => {
            panic!("Could not determine data directory")
        }
    };

    debug!("App data dir: {}", dir.display());

    std::fs::create_dir_all(&dir).expect("Failed to create data dir");

    let token_path = dir.join("auth.json");

    let file_content = fs::read_to_string(&token_path);

    match file_content {
        Ok(token) => {
            let token_data: TokenData =
                serde_json::from_str(&token).expect("auth.json is not in proper format");
            Some(token_data)
        }
        Err(_) => None,
    }
}

pub fn save_token_data_file(token_data: &TokenData) {
    let dir = match app_data_dir() {
        Some(path) => path,
        None => {
            panic!("Could not determine data directory")
        }
    };

    // Create the directory if needed
    std::fs::create_dir_all(&dir).expect("Failed to create data dir");

    let token_path = dir.join("auth.json");

    let json_content =
        serde_json::to_string_pretty(token_data).expect("Failed to serialize token data");

    fs::write(&token_path, json_content).expect("Failed to write token data to file");

    debug!("Token data saved to {}", token_path.display());
}

fn create_config_template(conf_path: &PathBuf) {
    let template = r#"# Auox Configuration File
# Add your SpareBank 1 API credentials below

client_id = "your-client-id-here"
client_secret = "your-client-secret-here"

# Your financial institution ID
# Examples: fid-smn (SpareBank 1 Midt-Norge), fid-snn (SpareBank 1 SR-Bank), etc.
financial_institution = "fid-smn"
"#;
    fs::write(conf_path, template).expect("Failed to create config.toml template");

    panic!(
        "\n\n[Auox] Config file created at: {}\n\
                Please edit this file and add your SpareBank 1 API credentials.\n\
                Then run Auox again.\n",
        conf_path.display()
    );
}
