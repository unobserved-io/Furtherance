// jaCounter - Keep track of JustAnswer Expert earnings
// Copyright (C) 2024 Ricky Kresslein <ricky@unobserved.io>

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FurSettings {
    pub database_url: String,
}

impl Default for FurSettings {
    fn default() -> Self {
        let db_url: PathBuf = get_default_db_path();

        FurSettings {
            database_url: db_url.to_string_lossy().into_owned(),
        }
    }
}

impl FurSettings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        let config_path: PathBuf = get_settings_path();
        let path_str = config_path.to_string_lossy().into_owned();

        // Check if the configuration file exists
        if config_path.exists() {
            builder = builder.add_source(File::with_name(&path_str));
        } else {
            // Create the default configuration file
            let default_settings = FurSettings::default();
            let toml =
                toml::to_string(&default_settings).expect("Failed to serialize default settings");
            fs::write(config_path, &toml).expect("Failed to write default config file");

            builder = builder.add_source(File::from_str(&toml, config::FileFormat::Toml));
        }

        let config = builder.build()?;
        config.try_deserialize()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let toml = toml::to_string(self).expect("Failed to serialize settings");
        fs::write(get_settings_path(), toml)
    }

    // Change the database_url and save the settings
    pub fn change_db_url(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.database_url = path.to_owned();
        self.save()
    }
}

pub fn get_data_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("io", "unobserved", "furtherance") {
        let path = PathBuf::from(proj_dirs.data_dir());
        create_dir_all(&path).expect("Unable to create data directory");

        path
    } else {
        PathBuf::new()
    }
}

pub fn get_default_db_path() -> PathBuf {
    let mut path = get_data_path();
    path.extend(&["furtherance.db"]);
    path
}

fn get_settings_path() -> PathBuf {
    let mut path = get_data_path();
    path.extend(&["settings.toml"]);
    path
}
