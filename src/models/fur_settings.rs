// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::view_enums::FurView;

use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FurSettings {
    pub database_url: String,
    pub default_view: FurView,
    pub notify_on_idle: bool,
    pub chosen_idle_time: i64,
    pub pomodoro: bool,
    pub pomodoro_break_length: i64,
    pub pomodoro_length: i64,
    pub pomodoro_snooze_length: i64,
}

impl Default for FurSettings {
    fn default() -> Self {
        let db_url: PathBuf = get_default_db_path();

        FurSettings {
            database_url: db_url.to_string_lossy().into_owned(),
            default_view: FurView::Timer,
            notify_on_idle: true,
            chosen_idle_time: 6,
            pomodoro: false,
            pomodoro_break_length: 5,
            pomodoro_length: 25,
            pomodoro_snooze_length: 5,
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
    pub fn change_db_url(&mut self, value: &str) -> Result<(), std::io::Error> {
        self.database_url = value.to_owned();
        self.save()
    }

    pub fn change_default_view(&mut self, value: &FurView) -> Result<(), std::io::Error> {
        self.default_view = value.to_owned();
        self.save()
    }

    pub fn change_notify_on_idle(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.notify_on_idle = value.to_owned();
        self.save()
    }

    pub fn change_chosen_idle_time(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.chosen_idle_time = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.pomodoro = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_break_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_break_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_snooze_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_snooze_length = value.to_owned();
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
