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
use serde::{Deserialize, Serialize};

use super::fur_task::FurTask;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskToEdit {
    pub id: u32,
    pub name: String,
    pub new_name: String,
    pub start_time: DateTime<Local>,
    pub new_start_time: DateTime<Local>,
    pub stop_time: DateTime<Local>,
    pub new_stop_time: DateTime<Local>,
    pub tags: String,
    pub new_tags: String,
    pub project: String,
    pub new_project: String,
    pub rate: f32,
    pub new_rate: String,
}

impl TaskToEdit {
    pub fn new_from(task: FurTask) -> Self {
        TaskToEdit {
            id: task.id,
            name: task.name.clone(),
            new_name: task.name.clone(),
            start_time: task.start_time,
            new_start_time: task.start_time,
            stop_time: task.stop_time,
            new_stop_time: task.stop_time,
            tags: task.tags.clone(),
            new_tags: task.tags.clone(),
            project: task.project.clone(),
            new_project: task.project.clone(),
            rate: task.rate,
            new_rate: task.rate.to_string(),
        }
    }
}
