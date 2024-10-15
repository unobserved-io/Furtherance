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

use std::{
    fs::{remove_file, File},
    io::{BufRead, BufReader, BufWriter, Result, Write},
    path::PathBuf,
};

use chrono::{DateTime, Local};

use crate::{
    app::split_task_input,
    database::db_write_task,
    models::{fur_settings::get_data_path, fur_task::FurTask},
};

pub fn autosave_exists() -> bool {
    let path = get_autosave_path();
    path.exists()
}

pub fn restore_autosave() {
    let path = get_autosave_path();
    if let Some(task) = task_from_autosave(&path) {
        if let Err(e) = db_write_task(task) {
            eprintln!("Error writing autosave to database: {e}");
        }

        delete_autosave();
    }
}

pub fn write_autosave(task_input: &str, start_time: DateTime<Local>) -> Result<()> {
    let start_time = start_time.to_rfc3339();
    let stop_time = Local::now().to_rfc3339();

    let path = get_autosave_path();
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let (name, project, tags, rate) = split_task_input(task_input);
    let currency = String::new();

    writeln!(writer, "{name}")?;
    writeln!(writer, "{start_time}")?;
    writeln!(writer, "{stop_time}")?;
    writeln!(writer, "{tags}")?;
    writeln!(writer, "{project}")?;
    writeln!(writer, "{rate}")?;
    writeln!(writer, "{currency}")?;

    Ok(())
}

pub fn delete_autosave() {
    let path = get_autosave_path();
    if path.exists() {
        if let Err(e) = remove_file(path) {
            eprintln!("Error deleting autosave: {e}");
        }
    }
}

fn get_autosave_path() -> PathBuf {
    let mut path = get_data_path();
    path.extend(&["autosave.txt"]);
    path
}

fn read_autosave(path: &PathBuf) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut vars = Vec::new();

    for line in reader.lines() {
        vars.push(line?);
    }

    Ok(vars)
}

fn task_from_autosave(path: &PathBuf) -> Option<FurTask> {
    if let Ok(autosave_lines) = read_autosave(path) {
        if let Ok(start_time) = DateTime::parse_from_rfc3339(&autosave_lines[1]) {
            if let Ok(stop_time) = DateTime::parse_from_rfc3339(&autosave_lines[2]) {
                let currency: String;
                if autosave_lines.len() < 7 {
                    currency = String::new();
                } else {
                    currency = autosave_lines[6].clone()
                }
                return Some(FurTask {
                    id: 0,
                    name: autosave_lines[0].clone(),
                    start_time: DateTime::from(start_time),
                    stop_time: DateTime::from(stop_time),
                    tags: autosave_lines[3].clone(),
                    project: autosave_lines[4].clone(),
                    rate: autosave_lines[5].parse().unwrap_or(0.0),
                    currency,
                    last_updated: chrono::Utc::now().timestamp(),
                });
            }
        }

        return None;
    } else {
        return None;
    }
}
