use serde::Deserialize;
use std::{fs, path::PathBuf};

fn app_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("aurox"))
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
            .expect("could not read example.toml")
            .iter()
            .map(|c| *c as char)
            .collect::<String>();

        let appconfig: AppConfig =
            toml::from_str(&file).expect("example.toml is not in proper format");

        appconfig
    } else {
        panic!("Unable to load config. Panicing....");
    }
}
