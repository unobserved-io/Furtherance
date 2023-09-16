// Furtherance - Track your time without being tracked
// Copyright (C) 2022  Ricky Kresslein <rk@lakoliu.com>
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

use chrono::{DateTime, Local};
use directories::ProjectDirs;
use gettextrs::*;
use glib::clone;
use gtk::prelude::*;
use gtk::glib;
use rusqlite::{Connection, Result, backup};
use std::convert::TryFrom;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::Duration;

use crate::ui::FurtheranceWindow;
use crate::settings_manager;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: i32,
    pub task_name: String,
    pub start_time: String,
    pub stop_time: String,
    pub tags: String,
}

impl ToString for Task {
    fn to_string(&self) -> String {
        format!("{} #{}", self.task_name, self.tags)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
pub enum SortOrder {
    Ascending = 0,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        // matches the default in sqlite
        Self::Ascending
    }
}

impl TryFrom<u32> for SortOrder {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .ok_or_else(|| anyhow::anyhow!("SortOrder from_u32() failed for value {}", value))
    }
}

impl SortOrder {
    fn to_sqlite(&self) -> &str {
        match self {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    num_derive::ToPrimitive,
    num_derive::FromPrimitive,
)]
pub enum TaskSort {
    StartTime = 0,
    StopTime,
    TaskName,
}

impl Default for TaskSort {
    fn default() -> Self {
        Self::StartTime
    }
}

impl TryFrom<u32> for TaskSort {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .ok_or_else(|| anyhow::anyhow!("TaskSort from_u32() failed for value {}", value))
    }
}

impl TaskSort {
    fn to_sqlite(&self) -> &str {
        match self {
            Self::StartTime => "start_time",
            Self::StopTime => "stop_time",
            Self::TaskName => "task_name",
        }
    }
}

pub fn get_directory() -> PathBuf {
    let dir_from_settings = settings_manager::get_string("database-loc");

    if dir_from_settings != "default" && PathBuf::from(dir_from_settings.clone()).exists() {
        return PathBuf::from(dir_from_settings);
    } else {
        if let Some(proj_dirs) = ProjectDirs::from("com", "lakoliu", "Furtherance") {
            let mut path = PathBuf::from(proj_dirs.data_dir());
            create_dir_all(path.clone()).expect("Unable to create database directory");
            path.extend(&["furtherance.db"]);

            let path_str = path.to_string_lossy().to_string();
            if path_str != dir_from_settings {
                let settings = settings_manager::get_settings();
                let _ = settings.set_string("database-loc", &path_str);
            }

            return path;
        }
    }

    PathBuf::new()
}

pub fn db_init() -> Result<()> {
    let conn = Connection::open(get_directory())?;
    conn.execute(
        "CREATE TABLE tasks (
                    id integer primary key,
                    task_name text,
                    start_time timestamp,
                    stop_time timestamp,
                    tags text)",
        [],
    )?;

    Ok(())
}

pub fn upgrade_old_db() -> Result<()> {
    // Update from old DB w/o tags
    let conn = Connection::open(get_directory())?;

    conn.execute("ALTER TABLE tasks ADD COLUMN tags TEXT DEFAULT ' '", [])?;

    Ok(())
}

pub fn db_write(
    task_name: &str,
    start_time: DateTime<Local>,
    stop_time: DateTime<Local>,
    tags: String,
) -> Result<()> {
    // Write data into database
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT INTO tasks (task_name, start_time, stop_time, tags) values (?1, ?2, ?3, ?4)",
        &[
            &task_name.to_string(),
            &start_time.to_rfc3339(),
            &stop_time.to_rfc3339(),
            &tags,
        ],
    )?;

    Ok(())
}

pub fn write_autosave(
    task_name: &str,
    start_time: &str,
    stop_time: &str,
    tags: &str,
) -> Result<()> {
    // Write data into database
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT INTO tasks (task_name, start_time, stop_time, tags) values (?1, ?2, ?3, ?4)",
        &[&task_name, &start_time, &stop_time, &tags],
    )?;

    Ok(())
}

pub fn retrieve(sort: TaskSort, order: SortOrder) -> Result<Vec<Task>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(get_directory())?;

    let mut query = conn.prepare(
        format!(
            "SELECT * FROM tasks ORDER BY {0} {1}",
            sort.to_sqlite(),
            order.to_sqlite()
        )
        .as_str(),
    )?;
    let task_iter = query.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            task_name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
        })
    })?;

    let mut tasks_vec: Vec<Task> = Vec::new();
    for task_item in task_iter {
        tasks_vec.push(task_item.unwrap());
    }

    Ok(tasks_vec)
}

/// Exports the database as CSV.
/// The delimiter parameter is interpreted as a ASCII character.
pub fn export_as_csv(sort: TaskSort, order: SortOrder, delimiter: u8) -> anyhow::Result<String> {
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    struct CSVTask {
        pub id: i32,
        pub task_name: String,
        pub start_time: String,
        pub stop_time: String,
        pub tags: String,
        pub seconds: i64,
    }

    let mut csv_writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(vec![]);
    let tasks = retrieve(sort, order)?;

    for task in tasks {
        let start_time = DateTime::parse_from_rfc3339(&task.start_time).unwrap();
        let stop_time = DateTime::parse_from_rfc3339(&task.stop_time).unwrap();
        let duration = stop_time - start_time;
        csv_writer.serialize(CSVTask{
            id: task.id,
            task_name: task.task_name,
            start_time: task.start_time,
            stop_time: task.stop_time,
            tags: task.tags,
            seconds: duration.num_seconds(),
        })?;
    }

    csv_writer.flush()?;

    Ok(String::from_utf8(csv_writer.into_inner()?)?)
}

pub fn update_start_time(id: i32, start_time: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET start_time = (?1) WHERE id = (?2)",
        &[&start_time, &id.to_string()],
    )?;

    Ok(())
}

pub fn update_stop_time(id: i32, stop_time: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET stop_time = (?1) WHERE id = (?2)",
        &[&stop_time, &id.to_string()],
    )?;

    Ok(())
}

pub fn update_task_name(id: i32, task_name: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET task_name = (?1) WHERE id = (?2)",
        &[&task_name, &id.to_string()],
    )?;

    Ok(())
}

pub fn update_tags(id: i32, tags: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET tags = (?1) WHERE id = (?2)",
        &[&tags, &id.to_string()],
    )?;

    Ok(())
}

pub fn get_list_by_id(id_list: Vec<i32>) -> Result<Vec<Task>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;
    let mut tasks_vec: Vec<Task> = Vec::new();

    for id in id_list {
        let mut query = conn.prepare("SELECT * FROM tasks WHERE id = :id;")?;
        let task_iter = query.query_map(&[(":id", &id.to_string())], |row| {
            Ok(Task {
                id: row.get(0)?,
                task_name: row.get(1)?,
                start_time: row.get(2)?,
                stop_time: row.get(3)?,
                tags: row.get(4)?,
            })
        })?;

        for task_item in task_iter {
            tasks_vec.push(task_item.unwrap());
        }
    }

    Ok(tasks_vec)
}

pub fn get_list_by_name(task_name: String) -> Result<Vec<Task>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;
    let mut tasks_vec: Vec<Task> = Vec::new();
    let name_param = format!("%{}%", task_name);

    let mut query = conn.prepare("SELECT * FROM tasks WHERE lower(task_name) LIKE lower(:task_name) ORDER BY task_name")?;
    let task_iter = query.query_map(&[(":task_name", &name_param)], |row| {
        Ok(Task {
            id: row.get(0)?,
            task_name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
        })
    })?;

    for task_item in task_iter {
        tasks_vec.push(task_item.unwrap());
    }

    Ok(tasks_vec)
}

pub fn check_for_tasks() -> Result<String> {
    let conn = Connection::open(get_directory())?;

    conn.query_row(
        "SELECT task_name FROM tasks ORDER BY ROWID ASC LIMIT 1",
        [],
        |row| row.get(0),
    )
}

pub fn check_db_validity(db_path: String) -> Result<String> {
    let conn = Connection::open(db_path)?;

    conn.query_row(
        "SELECT task_name FROM tasks ORDER BY ROWID ASC LIMIT 1",
        [],
        |row| row.get(0),
    )
}

pub fn delete_by_ids(id_list: Vec<i32>) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    for id in id_list {
        conn.execute("delete FROM tasks WHERE id = (?1)", &[&id.to_string()])?;
    }

    Ok(())
}

pub fn delete_by_id(id: i32) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute("delete FROM tasks WHERE id = (?1)", &[&id.to_string()])?;

    Ok(())
}

pub fn delete_all() -> Result<()> {
    // Delete everything from the database
    let conn = Connection::open(get_directory())?;

    conn.execute("delete from tasks", [])?;

    Ok(())
}

pub fn backup_db(backup_file: String) -> Result<()> {
    let mut bkup_conn = Connection::open(backup_file)?;
    let conn = Connection::open(get_directory())?;
    let backup = backup::Backup::new(&conn, &mut bkup_conn)?;
    backup.run_to_completion(5, Duration::from_millis(250), None)
}

pub fn import_db(new_db: String) -> Result<()> {
    let new_conn = Connection::open(new_db.clone())?;
    let valid = match check_db_validity(new_db) {
        Ok(_) => true,
        Err(_) => false
    };

    if valid {
        let mut conn = Connection::open(get_directory())?;
        let backup = backup::Backup::new(&new_conn, &mut conn)?;
        backup.run_to_completion(5, Duration::from_millis(250), None)
    } else {
        let window = FurtheranceWindow::default();
        let dialog = gtk::MessageDialog::with_markup(
            Some(&window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Ok,
            Some(&format!(
                "<span size='large' weight='bold'>{}</span>",
                &gettext("Not a valid database")
            )),
        );

        dialog.connect_response(clone!(@weak dialog = > move |_, _| {
            dialog.close();
        }));

        dialog.show();

        Ok(())
    }
}
