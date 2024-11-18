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

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurTask {
    pub name: String,
    pub start_time: DateTime<Local>,
    pub stop_time: DateTime<Local>,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub currency: String,
    pub uid: String,
    pub is_deleted: bool,
    pub last_updated: i64,
}

impl ToString for FurTask {
    fn to_string(&self) -> String {
        let mut task_string: String = self.name.to_string();

        if !self.project.is_empty() {
            task_string += &format!(" @{}", self.project);
        }
        if !self.tags.is_empty() {
            task_string += &format!(" #{}", self.tags);
        }
        if self.rate != 0.0 {
            task_string += &format!(" ${:.2}", self.rate);
        }

        task_string
    }
}

impl FurTask {
    pub fn new(
        name: String,
        start_time: DateTime<Local>,
        stop_time: DateTime<Local>,
        tags: String,
        project: String,
        rate: f32,
        currency: String,
    ) -> Self {
        let uid = generate_task_uid(&name, &start_time, &stop_time);

        FurTask {
            name,
            start_time,
            stop_time,
            tags,
            project,
            rate,
            currency,
            uid,
            is_deleted: false,
            last_updated: Utc::now().timestamp(),
        }
    }

    pub fn new_with_last_updated(
        name: String,
        start_time: DateTime<Local>,
        stop_time: DateTime<Local>,
        tags: String,
        project: String,
        rate: f32,
        currency: String,
        last_updated: i64,
    ) -> Self {
        let uid = generate_task_uid(&name, &start_time, &stop_time);

        FurTask {
            name,
            start_time,
            stop_time,
            tags,
            project,
            rate,
            currency,
            uid,
            is_deleted: false,
            last_updated,
        }
    }

    pub fn total_time_in_seconds(&self) -> i64 {
        (self.stop_time - self.start_time).num_seconds()
    }

    pub fn total_earnings(&self) -> f32 {
        (self.total_time_in_seconds() as f32 / 3600.0) * self.rate
    }
}

pub fn generate_task_uid(
    name: &str,
    start_time: &DateTime<Local>,
    stop_time: &DateTime<Local>,
) -> String {
    let input = format!(
        "{}{}{}",
        name,
        start_time.timestamp(),
        stop_time.timestamp()
    );

    blake3::hash(input.as_bytes()).to_hex().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTask {
    pub encrypted_data: String,
    pub nonce: String,
    pub uid: String,
    pub last_updated: i64,
}
