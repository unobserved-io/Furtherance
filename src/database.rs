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

use chrono::{DateTime, Local};
use directories::ProjectDirs;
use rusqlite::{backup, params, Connection, Result};
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::Duration;

use crate::models::{fur_settings::FurSettings, fur_task::FurTask};

#[derive(Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
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

#[derive(Debug)]
pub enum SortBy {
    StartTime,
    StopTime,
    TaskName,
}

impl Default for SortBy {
    fn default() -> Self {
        Self::StartTime
    }
}

impl SortBy {
    fn to_sqlite(&self) -> &str {
        match self {
            Self::StartTime => "start_time",
            Self::StopTime => "stop_time",
            Self::TaskName => "task_name",
        }
    }
}

pub fn db_get_directory() -> PathBuf {
    // Get DB location from settings
    let settings_db_dir = match FurSettings::new() {
        Ok(loaded_settings) => loaded_settings.database_url,
        Err(e) => {
            eprintln!("Error loading settings: {}", e);
            FurSettings::default().database_url
        }
    };

    PathBuf::from(&settings_db_dir)
}

pub fn db_init() -> Result<()> {
    let conn = Connection::open(db_get_directory())?;
    conn.execute(
        "CREATE TABLE tasks (
                    id INTEGER PRIMARY KEY,
                    task_name TEXT,
                    start_time TIMESTAMP,
                    stop_time TIMESTAMP,
                    tags TEXT,
                    project TEXT,
                    rate REAL)",
        [],
    )?;

    Ok(())
}

pub fn db_upgrade_old() -> Result<()> {
    // Update from old DB w/o tags, project, or rates
    let conn = Connection::open(db_get_directory())?;

    let _ = db_add_tags_column(&conn);
    let _ = db_add_project_column(&conn);
    let _ = db_add_rate_column(&conn);

    Ok(())
}

pub fn db_add_tags_column(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE tasks ADD COLUMN tags TEXT DEFAULT ''", [])?;
    Ok(())
}

pub fn db_add_project_column(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE tasks ADD COLUMN project TEXT DEFAULT ''", [])?;
    Ok(())
}

pub fn db_add_rate_column(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE tasks ADD COLUMN rate REAL DEFAULT 0.0", [])?;
    Ok(())
}

pub fn db_write(fur_task: FurTask) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "INSERT INTO tasks (
            task_name,
            start_time,
            stop_time,
            tags,
            project,
            rate
        ) values (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            fur_task.name,
            fur_task.start_time.to_rfc3339(),
            fur_task.stop_time.to_rfc3339(),
            fur_task.tags,
            fur_task.project,
            fur_task.rate,
        ],
    )?;

    Ok(())
}

pub fn db_retrieve_all(sort: SortBy, order: SortOrder) -> Result<Vec<FurTask>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(db_get_directory())?;

    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM tasks ORDER BY {0} {1}",
            sort.to_sqlite(),
            order.to_sqlite()
        )
        .as_str(),
    )?;
    let mut rows = stmt.query(params![])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            id: row.get(0)?,
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn update_start_time(id: i32, start_time: String) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET start_time = (?1) WHERE id = (?2)",
        params![start_time, id],
    )?;

    Ok(())
}

pub fn update_stop_time(id: i32, stop_time: String) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET stop_time = (?1) WHERE id = (?2)",
        params![stop_time, id],
    )?;

    Ok(())
}

pub fn update_task_name(id: i32, task_name: String) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET task_name = (?1) WHERE id = (?2)",
        params![task_name, id],
    )?;

    Ok(())
}

pub fn update_tags(id: i32, tags: String) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET tags = (?1) WHERE id = (?2)",
        params![tags, id],
    )?;

    Ok(())
}

pub fn update_project(id: i32, project: String) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET project = (?1) WHERE id = (?2)",
        params![project, id],
    )?;

    Ok(())
}

pub fn update_rate(id: i32, rate: f32) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET rate = (?1) WHERE id = (?2)",
        params![rate, id],
    )?;

    Ok(())
}

pub fn get_list_by_id(id_list: Vec<i32>) -> Result<Vec<FurTask>, rusqlite::Error> {
    let conn = Connection::open(db_get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?")?;
    let mut tasks_vec = Vec::new();

    for id in id_list {
        let task_iter = stmt.query_map(&[&id], |row| {
            Ok(FurTask {
                id: row.get(0)?,
                name: row.get(1)?,
                start_time: row.get(2)?,
                stop_time: row.get(3)?,
                tags: row.get(4)?,
                project: row.get(5)?,
                rate: row.get(6)?,
            })
        })?;

        for task_item in task_iter {
            tasks_vec.push(task_item?);
        }
    }

    Ok(tasks_vec)
}

pub fn get_list_by_name_and_tags(
    task_name: String,
    tag_list: Vec<String>,
) -> Result<Vec<FurTask>, rusqlite::Error> {
    let conn = Connection::open(db_get_directory())?;

    let name_param = format!("%{}%", task_name);
    let tag_list_params: Vec<String> = tag_list.iter().map(|tag| format!("%{}%", tag)).collect();

    let mut sql_query = String::from("SELECT * FROM tasks WHERE lower(task_name) LIKE lower(?)");
    tag_list_params
        .iter()
        .for_each(|_| sql_query.push_str(" AND lower(tags) LIKE lower(?)"));
    sql_query.push_str(" ORDER BY task_name");

    let mut query = conn.prepare(sql_query.as_str())?;
    query.raw_bind_parameter(1, name_param)?;
    for (i, tag) in tag_list_params.iter().enumerate() {
        query.raw_bind_parameter(i + 2, tag)?;
    }

    let tasks_vec = query
        .raw_query()
        .mapped(|row| {
            Ok(FurTask {
                id: row.get(0)?,
                name: row.get(1)?,
                start_time: row.get(2)?,
                stop_time: row.get(3)?,
                tags: row.get(4)?,
                project: row.get(5)?,
                rate: row.get(6)?,
            })
        })
        .map(|task_item| task_item.unwrap())
        .collect();

    Ok(tasks_vec)
}

pub fn check_for_tasks() -> Result<String> {
    let conn = Connection::open(db_get_directory())?;

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
    let conn = Connection::open(db_get_directory())?;

    for id in id_list {
        conn.execute("delete FROM tasks WHERE id = (?1)", &[&id.to_string()])?;
    }

    Ok(())
}

pub fn delete_by_id(id: i32) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute("delete FROM tasks WHERE id = (?1)", &[&id.to_string()])?;

    Ok(())
}

pub fn delete_all() -> Result<()> {
    // Delete everything from the database
    let conn = Connection::open(db_get_directory())?;

    conn.execute("delete from tasks", [])?;

    Ok(())
}

pub fn backup_db(backup_file: String) -> Result<()> {
    let mut bkup_conn = Connection::open(backup_file)?;
    let conn = Connection::open(db_get_directory())?;
    let backup = backup::Backup::new(&conn, &mut bkup_conn)?;
    backup.run_to_completion(5, Duration::from_millis(250), None)
}

pub fn import_db(new_db: String) -> Result<()> {
    let new_conn = Connection::open(new_db.clone())?;
    let valid = match check_db_validity(new_db) {
        Ok(_) => true,
        Err(_) => false,
    };

    if valid {
        let mut conn = Connection::open(db_get_directory())?;
        let backup = backup::Backup::new(&new_conn, &mut conn)?;
        backup.run_to_completion(5, Duration::from_millis(250), None)
    } else {
        // TODO: Show error
        Ok(())
    }
}
