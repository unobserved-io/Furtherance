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

use rusqlite::{Connection, Result};
use chrono::{DateTime, Local};
use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs::create_dir_all;

#[derive(Clone, Debug)]
pub struct Task {
    pub id: i32,
    pub task_name: String,
    pub start_time: String,
    pub stop_time: String,
}

pub fn get_directory() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "lakoliu",  "Furtherance") {
        let mut path = PathBuf::from(proj_dirs.data_dir());
        create_dir_all(path.clone()).expect("Unable to create database directory");
        path.extend(&["furtherance.db"]);
        return path
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
                    stop_time timestamp)",
        [],
    )?;

    Ok(())
}


pub fn db_write(task_name: &str, start_time: DateTime<Local>, stop_time: DateTime<Local>) -> Result<()> {
    // Write data into database
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT INTO tasks (task_name, start_time, stop_time) values (?1, ?2, ?3)",
        &[&task_name.to_string(), &start_time.to_rfc3339(), &stop_time.to_rfc3339()],
    )?;

    Ok(())
}

pub fn retrieve() -> Result<Vec<Task>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(get_directory())?;

    let mut query = conn.prepare("SELECT * FROM tasks ORDER BY start_time")?;
    let task_iter = query.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            task_name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
        })
    })?;

    let mut tasks_vec: Vec<Task> = Vec::new();
    for task_item in task_iter {
        tasks_vec.push(task_item.unwrap());
    }

    Ok(tasks_vec)

}

pub fn update_start_time(id: i32, start_time: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET start_time = (?1) WHERE id = (?2)",
        &[&start_time, &id.to_string()]
    )?;

    Ok(())
}

pub fn update_stop_time(id: i32, stop_time: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET stop_time = (?1) WHERE id = (?2)",
        &[&stop_time, &id.to_string()]
    )?;

    Ok(())
}

pub fn update_task_name(id: i32, task_name: String) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET task_name = (?1) WHERE id = (?2)",
        &[&task_name, &id.to_string()]
    )?;

    Ok(())
}

pub fn get_list_by_id(id_list: Vec<i32>) -> Result<Vec<Task>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;
    let mut tasks_vec: Vec<Task> = Vec::new();

    for id in id_list {
        let mut query = conn.prepare(
            "SELECT * FROM tasks WHERE id = :id;")?;
        let task_iter = query.query_map(&[(":id", &id.to_string())], |row| {
            Ok(Task {
                id: row.get(0)?,
                task_name: row.get(1)?,
                start_time: row.get(2)?,
                stop_time: row.get(3)?,
            })
        })?;

        for task_item in task_iter {
            tasks_vec.push(task_item.unwrap());
        }
    }

    Ok(tasks_vec)
}

pub fn check_for_tasks() -> Result<String> {
    let conn = Connection::open(get_directory())?;

    conn.query_row(
        "SELECT task_name FROM tasks WHERE id='1'",
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

    conn.execute("delete from tasks",[],)?;

    Ok(())
}
