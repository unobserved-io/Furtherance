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

use chrono::offset::LocalResult;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;
use chrono::TimeZone;
use rusqlite::{backup, params, Connection, Result};
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crate::models::fur_shortcut;
use crate::models::fur_task;
use crate::models::fur_user::FurUser;
use crate::models::{
    fur_settings::FurSettings, fur_shortcut::FurShortcut, fur_task::FurTask,
    group_to_edit::GroupToEdit,
};
#[cfg(target_os = "macos")]
use crate::view_enums::FurAlert;

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

#[allow(dead_code)]
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

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!(
        "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name = ?",
        table
    ))?;
    let count: i64 = stmt.query_row([column], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn db_init() -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            task_name TEXT,
            start_time TIMESTAMP,
            stop_time TIMESTAMP,
            tags TEXT,
            project TEXT,
            rate REAL,
            currency TEXT,
            uid TEXT,
            is_deleted BOOLEAN DEFAULT 0,
            last_updated INTEGER DEFAULT 0
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS shortcuts (
            id INTEGER PRIMARY KEY,
            name TEXT,
            tags TEXT,
            project TEXT,
            rate REAL,
            currency TEXT,
            color_hex TEXT,
            uid TEXT,
            is_deleted BOOLEAN DEFAULT 0,
            last_updated INTEGER DEFAULT 0
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            email TEXT PRIMARY KEY,
            encrypted_key TEXT NOT NULL,
            key_nonce TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            server TEXT NOT NULL
        )",
        [],
    )?;

    db_upgrade_old()?;

    Ok(())
}

pub fn db_upgrade_old() -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    if !column_exists(&conn, "tasks", "tags")? {
        db_add_tags_column(&conn)?;
    }
    if !column_exists(&conn, "tasks", "project")? {
        db_add_project_column(&conn)?;
    }
    if !column_exists(&conn, "tasks", "rate")? {
        db_add_rate_column(&conn)?;
    }
    if !column_exists(&conn, "tasks", "currency")? {
        db_add_currency_column(&conn)?;
    }
    if !column_exists(&conn, "tasks", "uid")? {
        db_add_sync_columns(&conn)?;
    }

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

pub fn db_add_currency_column(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;
        ALTER TABLE tasks ADD COLUMN currency Text DEFAULT ''
        UPDATE tasks SET currency = '' WHERE currency IS NULL;
        COMMIT;",
    )?;
    Ok(())
}

pub fn db_add_sync_columns(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "tasks", "uid")? {
        conn.execute("ALTER TABLE tasks ADD COLUMN uid TEXT", [])?;
        let mut stmt = conn
            .prepare("SELECT id, task_name, start_time, stop_time FROM tasks WHERE uid IS NULL")?;
        let tasks: Vec<(i64, String, DateTime<Local>, DateTime<Local>)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for (id, name, start_time, stop_time) in tasks {
            let uid = fur_task::generate_task_uid(&name, &start_time, &stop_time);
            conn.execute("UPDATE tasks SET uid = ?1 WHERE id = ?2", params![uid, id])?;
        }
    }

    if !column_exists(conn, "shortcuts", "uid")? {
        conn.execute("ALTER TABLE shortcuts ADD COLUMN uid TEXT", [])?;
        let mut stmt = conn.prepare(
            "SELECT id, name, tags, project, rate, currency FROM shortcuts WHERE uid IS NULL",
        )?;
        let shortcuts: Vec<(i64, String, String, String, f32, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for (id, name, tags, project, rate, currency) in shortcuts {
            let uid = fur_shortcut::generate_shortcut_uid(&name, &tags, &project, &rate, &currency);
            conn.execute(
                "UPDATE shortcuts SET uid = ?1 WHERE id = ?2",
                params![uid, id],
            )?;
        }
    }

    if !column_exists(conn, "tasks", "is_deleted")? {
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN is_deleted BOOLEAN DEFAULT 0",
            [],
        )?;
    }

    if !column_exists(conn, "shortcuts", "is_deleted")? {
        conn.execute(
            "ALTER TABLE shortcuts ADD COLUMN is_deleted BOOLEAN DEFAULT 0",
            [],
        )?;
    }

    if !column_exists(conn, "tasks", "last_updated")? {
        conn.execute(
            "ALTER TABLE tasks ADD COLUMN last_updated INTEGER DEFAULT 0",
            [],
        )?;
    }

    if !column_exists(conn, "shortcuts", "last_updated")? {
        conn.execute(
            "ALTER TABLE shortcuts ADD COLUMN last_updated INTEGER DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

pub fn db_insert_task(task: &FurTask) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "INSERT INTO tasks (
            task_name,
            start_time,
            stop_time,
            tags,
            project,
            rate,
            currency,
            uid,
            is_deleted,
            last_updated
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            task.name,
            task.start_time.to_rfc3339(),
            task.stop_time.to_rfc3339(),
            task.tags,
            task.project,
            task.rate,
            task.currency,
            task.uid,
            task.is_deleted,
            task.last_updated
        ],
    )?;

    Ok(())
}

pub fn db_insert_tasks(tasks: &[FurTask]) -> Result<()> {
    let mut conn = Connection::open(db_get_directory())?;

    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO tasks (
                task_name,
                start_time,
                stop_time,
                tags,
                project,
                rate,
                currency,
                uid,
                is_deleted,
                last_updated
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )?;

        for task in tasks {
            stmt.execute(params![
                task.name,
                task.start_time.to_rfc3339(),
                task.stop_time.to_rfc3339(),
                task.tags,
                task.project,
                task.rate,
                task.currency,
                task.uid,
                task.is_deleted,
                task.last_updated
            ])?;
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn db_retrieve_all_tasks(
    sort: SortBy,
    order: SortOrder,
) -> Result<Vec<FurTask>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(db_get_directory())?;

    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM tasks WHERE is_deleted = 0 ORDER BY {0} {1}",
            sort.to_sqlite(),
            order.to_sqlite()
        )
        .as_str(),
    )?;
    let mut rows = stmt.query(params![])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn db_retrieve_tasks_by_date_range(
    start_date: String,
    end_date: String,
) -> Result<Vec<FurTask>> {
    let conn = Connection::open(db_get_directory())?;
    let mut stmt = conn.prepare(
        "SELECT * FROM tasks WHERE start_time BETWEEN ?1 AND ?2 AND is_deleted = 0 ORDER BY start_time ASC",
    )?;
    let mut rows = stmt.query(params![start_date, end_date])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

/// Retrieve a limited number of days worth of tasks
pub fn db_retrieve_tasks_with_day_limit(
    days: i64,
    sort: SortBy,
    order: SortOrder,
) -> Result<Vec<FurTask>> {
    let conn = Connection::open(db_get_directory())?;

    // Construct the query string dynamically
    let query = format!(
        "SELECT * FROM tasks WHERE start_time >= date('now', ?) AND is_deleted = 0 ORDER BY {} {}",
        sort.to_sqlite(),
        order.to_sqlite()
    );

    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query(params![format!("-{} days", days - 1)])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn db_retrieve_task_by_id(uid: &String) -> Result<Option<FurTask>> {
    let conn = Connection::open(db_get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE uid = ?")?;
    let mut rows = stmt.query_map([uid.to_string()], |row| {
        Ok(FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        })
    })?;

    match rows.next() {
        Some(Ok(task)) => Ok(Some(task)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn db_update_task(task: &FurTask) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE tasks SET
            task_name = ?1,
            start_time = ?2,
            stop_time = ?3,
            tags = ?4,
            project = ?5,
            rate = ?6,
            currency = ?7,
            is_deleted = ?8,
            last_updated = ?9,
        WHERE uid = ?10",
        params![
            task.name,
            task.start_time.to_rfc3339(),
            task.stop_time.to_rfc3339(),
            task.tags,
            task.project,
            task.rate,
            task.currency,
            task.is_deleted,
            task.last_updated,
            task.uid,
        ],
    )?;

    Ok(())
}

pub fn db_update_group_of_tasks(group: &GroupToEdit) -> Result<()> {
    let mut conn = Connection::open(db_get_directory())?;
    // Transaction ensures all updates succeed or none do.
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "UPDATE tasks SET
            task_name = ?1,
            tags = ?2,
            project = ?3,
            rate = ?4,
            last_updated = ?5,
        WHERE uid = ?6",
        )?;

        for uid in group.all_task_ids().iter() {
            stmt.execute(params![
                group.new_name.trim(),
                group
                    .new_tags
                    .trim()
                    .strip_prefix('#')
                    .unwrap_or(&group.tags)
                    .trim()
                    .to_string(),
                group.new_project.trim(),
                group.new_rate.trim().parse::<f32>().unwrap_or(0.0),
                chrono::Utc::now().timestamp(),
                uid,
            ])?;
        }
    }

    // Commit the transaction
    tx.commit()?;

    Ok(())
}

pub fn db_task_exists(task: &FurTask) -> Result<bool> {
    let conn = Connection::open(db_get_directory())?;

    let query = "
        SELECT 1 FROM tasks
        WHERE task_name = ?1
        AND start_time = ?2
        AND stop_time = ?3
        AND tags = ?4
        AND project = ?5
        AND rate = ?6
        AND currency = ?7
        AND is_deleted = ?8
        LIMIT 1
    ";

    let mut stmt = conn.prepare(query)?;

    let exists = stmt.exists(params![
        task.name,
        task.start_time.to_rfc3339(),
        task.stop_time.to_rfc3339(),
        task.tags,
        task.project,
        task.rate,
        task.currency,
        task.is_deleted,
    ])?;

    Ok(exists)
}

pub fn db_delete_tasks_by_ids(id_list: &[String]) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    for id in id_list {
        conn.execute(
            "UPDATE tasks SET is_deleted = 1 WHERE uid = (?1)",
            &[&id.to_string()],
        )?;
    }

    Ok(())
}

/// Insert a shortcut to the database
pub fn db_insert_shortcut(shortcut: &FurShortcut) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;
    conn.execute(
        "INSERT INTO shortcuts (
            name,
            tags,
            project,
            rate,
            currency,
            color_hex,
            uid,
            is_deleted,
            last_updated
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            shortcut.name,
            shortcut.tags,
            shortcut.project,
            shortcut.rate,
            shortcut.currency,
            shortcut.color_hex,
            shortcut.uid,
            shortcut.is_deleted,
            shortcut.last_updated,
        ],
    )?;

    Ok(())
}

/// Retrieve all shortcuts from the database
pub fn db_retrieve_shortcuts() -> Result<Vec<FurShortcut>, rusqlite::Error> {
    let conn = Connection::open(db_get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM shortcuts WHERE is_deleted = 0 ORDER BY name")?;
    let mut rows = stmt.query(params![])?;

    let mut shortcuts: Vec<FurShortcut> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_shortcut = FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        };
        shortcuts.push(fur_shortcut);
    }

    Ok(shortcuts)
}

pub fn db_update_shortcut(shortcut: &FurShortcut) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE shortcuts SET
            name = (?1),
            tags = (?2),
            project = (?3),
            rate = (?4),
            currency = (?5),
            color_hex = (?6),
            is_deleted = (?7),
            last_updated = (?8)
        WHERE uid = (?9)",
        params![
            shortcut.name,
            shortcut.tags,
            shortcut.project,
            shortcut.rate,
            shortcut.currency,
            shortcut.color_hex,
            shortcut.is_deleted,
            shortcut.last_updated,
            shortcut.uid,
        ],
    )?;

    Ok(())
}

pub fn db_shortcut_exists(shortcut: &FurShortcut) -> Result<bool> {
    let conn = Connection::open(db_get_directory())?;

    let query = "
        SELECT 1 FROM shortcuts
        WHERE name = ?1
        AND tags = ?2
        AND project = ?3
        AND rate = ?4
        AND currency = ?5
        AND is_deleted = 0
        LIMIT 1
    ";

    let mut stmt = conn.prepare(query)?;

    let exists = stmt.exists(params![
        shortcut.name,
        shortcut.tags,
        shortcut.project,
        shortcut.rate,
        shortcut.currency,
    ])?;

    Ok(exists)
}

pub fn db_retrieve_shortcut_by_id(uid: &String) -> Result<Option<FurShortcut>> {
    let conn = Connection::open(db_get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM shortcuts WHERE uid = ?")?;
    let mut rows = stmt.query_map([uid.to_string()], |row| {
        Ok(FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        })
    })?;

    match rows.next() {
        Some(Ok(shortcut)) => Ok(Some(shortcut)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn db_delete_shortcut_by_id(uid: &str) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE shortcuts SET is_deleted = 1 WHERE uid = (?1)",
        &[&uid],
    )?;

    Ok(())
}

pub fn db_delete_everything() -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute_batch(
        "
            BEGIN TRANSACTION;
            UPDATE tasks SET is_deleted = 1;
            UPDATE shortcuts SET is_deleted = 1;
            COMMIT;
        ",
    )?;

    Ok(())
}

pub fn db_backup(backup_file: PathBuf) -> Result<()> {
    let mut bkup_conn = Connection::open(backup_file)?;
    let conn = Connection::open(db_get_directory())?;
    let backup = backup::Backup::new(&conn, &mut bkup_conn)?;
    backup.run_to_completion(5, Duration::from_millis(250), None)
}

pub fn db_retrieve_orphaned_tasks(task_uids: Vec<String>) -> Result<Vec<FurTask>> {
    let mut conn = Connection::open(db_get_directory())?;
    let mut tasks = Vec::new();

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("SELECT * FROM tasks WHERE uid = ?")?;

        for uid in task_uids {
            let task_iter = stmt.query_map(params![uid], |row| {
                Ok(FurTask {
                    name: row.get(1)?,
                    start_time: row.get(2)?,
                    stop_time: row.get(3)?,
                    tags: row.get(4)?,
                    project: row.get(5)?,
                    rate: row.get(6)?,
                    currency: row.get(7).unwrap_or(String::new()),
                    uid: row.get(8)?,
                    is_deleted: row.get(9)?,
                    last_updated: row.get(10)?,
                })
            })?;

            // Collect any matching tasks
            for task in task_iter {
                tasks.push(task?);
            }
        }
    }

    tx.commit()?;
    Ok(tasks)
}

pub fn db_retrieve_orphaned_shortcuts(shortcut_uids: Vec<String>) -> Result<Vec<FurShortcut>> {
    let mut conn = Connection::open(db_get_directory())?;
    let mut shortcuts = Vec::new();

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("SELECT * FROM shortcuts WHERE uid = ?")?;

        for uid in shortcut_uids {
            let shortcut_iter = stmt.query_map(params![uid], |row| {
                Ok(FurShortcut {
                    name: row.get(1)?,
                    tags: row.get(2)?,
                    project: row.get(3)?,
                    rate: row.get(4)?,
                    currency: row.get(5)?,
                    color_hex: row.get(6)?,
                    uid: row.get(7)?,
                    is_deleted: row.get(8)?,
                    last_updated: row.get(9)?,
                })
            })?;

            // Collect any matching tasks
            for shortcut in shortcut_iter {
                shortcuts.push(shortcut?);
            }
        }
    }

    tx.commit()?;
    Ok(shortcuts)
}

pub fn db_retrieve_tasks_since_timestamp(timestamp: i64) -> Result<Vec<FurTask>, rusqlite::Error> {
    let conn = Connection::open(db_get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM tasks WHERE last_updated >= ? ORDER BY last_updated ASC")?;
    let mut rows = stmt.query(params![timestamp])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn db_retrieve_shortcuts_since_timestamp(
    timestamp: i64,
) -> Result<Vec<FurShortcut>, rusqlite::Error> {
    let conn = Connection::open(db_get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM shortcuts WHERE last_updated >= ? ORDER BY last_updated ASC")?;
    let mut rows = stmt.query(params![timestamp])?;

    let mut shortcuts_vec: Vec<FurShortcut> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_shortcut = FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        };
        shortcuts_vec.push(fur_shortcut);
    }

    Ok(shortcuts_vec)
}

pub fn db_is_valid_v3(path: &Path) -> Result<bool> {
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(_) => return Ok(false),
    };

    // Check if the table 'tasks' exists
    let mut stmt =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'") {
            Ok(stmt) => stmt,
            Err(_) => return Ok(false),
        };
    let table_exists = match stmt.exists([]) {
        Ok(exists) => exists,
        Err(_) => return Ok(false),
    };
    if !table_exists {
        return Ok(false);
    }

    // Verify the table's structure
    let expected_columns = [
        "id integer",
        "task_name text",
        "start_time timestamp",
        "stop_time timestamp",
        "tags text",
        "project text",
        "rate real",
        "currency text",
    ];
    let mut stmt = match conn.prepare("PRAGMA table_info(tasks)") {
        Ok(stmt) => stmt,
        Err(_) => return Ok(false),
    };
    let column_info = match stmt.query_map([], |row| {
        Ok(format!(
            "{} {}",
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?.to_lowercase()
        ))
    }) {
        Ok(iter) => iter,
        Err(_) => return Ok(false),
    };

    let mut columns: Vec<String> = Vec::new();
    for column in column_info {
        match column {
            Ok(col) => columns.push(col),
            Err(_) => return Ok(false),
        }
    }
    for expected_col in expected_columns.iter() {
        if !columns.contains(&expected_col.to_string()) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn db_is_valid_v1(path: &Path) -> Result<bool> {
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(_) => return Ok(false),
    };

    // Check if the table 'tasks' exists
    let mut stmt =
        match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'") {
            Ok(stmt) => stmt,
            Err(_) => return Ok(false),
        };
    let table_exists = match stmt.exists([]) {
        Ok(exists) => exists,
        Err(_) => return Ok(false),
    };
    if !table_exists {
        return Ok(false);
    }

    // Verify the table's structure
    let expected_columns = [
        "id integer",
        "task_name text",
        "start_time timestamp",
        "stop_time timestamp",
        "tags text",
    ];
    let mut stmt = match conn.prepare("PRAGMA table_info(tasks)") {
        Ok(stmt) => stmt,
        Err(_) => return Ok(false),
    };
    let column_info = match stmt.query_map([], |row| {
        Ok(format!(
            "{} {}",
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?.to_lowercase()
        ))
    }) {
        Ok(iter) => iter,
        Err(_) => return Ok(false),
    };

    let mut columns: Vec<String> = Vec::new();
    for column in column_info {
        match column {
            Ok(col) => columns.push(col),
            Err(_) => return Ok(false),
        }
    }
    for expected_col in expected_columns.iter() {
        if !columns.contains(&expected_col.to_string()) {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(target_os = "macos")]
pub fn db_check_for_existing_mac_db() -> Option<FurAlert> {
    if let Some(user_dirs) = directories::UserDirs::new() {
        let mut path = user_dirs.home_dir().to_path_buf();
        path.extend(&[
            "Library",
            "Containers",
            "com.lakoliu.furtherance",
            "Data",
            "Library",
            "Application Support",
            "Furtherance",
            "Furtherance.sqlite",
        ]);
        if path.exists() {
            match db_mac_core_data_db_is_valid(&path) {
                Ok(is_valid) => {
                    if is_valid {
                        return Some(FurAlert::ImportMacDatabase);
                    }
                }
                Err(_) => return None,
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
pub fn db_mac_core_data_db_is_valid(path: &PathBuf) -> Result<bool> {
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return Ok(false);
        }
    };

    let mut stmt = match conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='ZFURTASK'")
    {
        Ok(stmt) => stmt,
        Err(_) => return Ok(false),
    };

    let table_exists = match stmt.exists([]) {
        Ok(exists) => exists,
        Err(_) => return Ok(false),
    };
    if !table_exists {
        return Ok(false);
    }

    let expected_columns = [
        "Z_PK integer",
        "Z_ENT integer",
        "Z_OPT integer",
        "ZSTARTTIME timestamp",
        "ZSTOPTIME timestamp",
        "ZNAME varchar",
        "ZTAGS varchar",
        "ZID blob",
        "ZRATE float",
        "ZPROJECT varchar",
    ];
    let mut stmt = match conn.prepare("PRAGMA table_info(ZFURTASK)") {
        Ok(stmt) => stmt,
        Err(_) => return Ok(false),
    };
    let column_info = match stmt.query_map([], |row| {
        Ok(format!(
            "{} {}",
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?.to_lowercase()
        ))
    }) {
        Ok(iter) => iter,
        Err(_) => return Ok(false),
    };

    let mut columns: Vec<String> = Vec::new();
    for column in column_info {
        match column {
            Ok(col) => columns.push(col),
            Err(_) => return Ok(false),
        }
    }
    for expected_col in expected_columns.iter() {
        if !columns.contains(&expected_col.to_string()) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn db_import_old_mac_db() -> Result<()> {
    if let Some(user_dirs) = directories::UserDirs::new() {
        let mut path = user_dirs.home_dir().to_path_buf();
        path.extend(&[
            "Library",
            "Containers",
            "com.lakoliu.furtherance",
            "Data",
            "Library",
            "Application Support",
            "Furtherance",
            "Furtherance.sqlite",
        ]);
        if path.exists() {
            let source_db = Connection::open(path)?;
            let mut stmt = source_db.prepare(
                "SELECT ZNAME, ZSTARTTIME, ZSTOPTIME, ZTAGS, ZPROJECT, ZRATE FROM ZFURTASK",
            )?;

            let mut rows = stmt.query(params![])?;

            let mut tasks_vec: Vec<FurTask> = Vec::new();

            while let Some(row) = rows.next()? {
                let fur_task = FurTask::new(
                    row.get(0)?,
                    core_data_timestamp_to_datetime(row.get(1)?)?,
                    core_data_timestamp_to_datetime(row.get(2)?)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    String::new(),
                );

                // Don't import duplicate tasks
                if let Ok(exists) = db_task_exists(&fur_task) {
                    if !exists {
                        tasks_vec.push(fur_task);
                    }
                }
            }

            db_insert_tasks(&tasks_vec)?;
        }
    }

    Ok(())
}

fn core_data_timestamp_to_datetime(timestamp: f64) -> Result<DateTime<Local>> {
    let seconds = timestamp.trunc() as i64;
    // Core Data reference date is January 1, 2001
    if let LocalResult::Single(core_data_epoch) = Local.with_ymd_and_hms(2001, 1, 1, 0, 0, 0) {
        let duration = TimeDelta::seconds(seconds);
        return Ok(core_data_epoch + duration);
    }
    return Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
        Some("Could not convert Core Data timestamp".to_string()),
    ));
}

pub fn db_retrieve_credentials() -> Result<Option<FurUser>> {
    let conn = Connection::open(db_get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM user LIMIT 1")?;

    let result = stmt.query_row([], |row| {
        Ok(FurUser {
            email: row.get(0)?,
            encrypted_key: row.get(1)?,
            key_nonce: row.get(2)?,
            access_token: row.get(3)?,
            refresh_token: row.get(4)?,
            server: row.get(5)?,
        })
    });

    match result {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn db_store_credentials(
    email: &str,
    encrypted_key: &str,
    key_nonce: &str,
    access_token: &str,
    refresh_token: &str,
    server: &str,
) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "INSERT OR REPLACE INTO user
        (email, encrypted_key, key_nonce, access_token, refresh_token, server)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            email,
            encrypted_key,
            key_nonce,
            access_token,
            refresh_token,
            server
        ],
    )?;

    Ok(())
}

pub fn db_update_access_token(email: &str, new_token: &str) -> Result<()> {
    let conn = Connection::open(db_get_directory())?;

    conn.execute(
        "UPDATE user
         SET access_token = ?1
         WHERE email = ?2",
        params![new_token, email],
    )?;

    Ok(())
}
