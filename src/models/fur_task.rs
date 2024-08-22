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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurTask {
    pub id: u32,
    pub name: String,
    pub start_time: DateTime<Local>,
    pub stop_time: DateTime<Local>,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub currency: String,
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
    pub fn total_time_in_seconds(&self) -> i64 {
        (self.stop_time - self.start_time).num_seconds()
    }
}
