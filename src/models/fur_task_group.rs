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

use crate::models::fur_task::FurTask;

use std::fmt;

#[derive(Debug, Clone)]
pub struct FurTaskGroup {
    pub id: u32,
    pub name: String,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub total_time: i64,
    pub tasks: Vec<FurTask>,
}

impl FurTaskGroup {
    pub fn new_from(task: FurTask) -> Self {
        FurTaskGroup {
            id: task.id,
            name: task.name.clone(),
            tags: task.tags.clone(),
            project: task.project.clone(),
            rate: task.rate,
            total_time: (task.stop_time - task.start_time).num_seconds(),
            tasks: vec![task],
        }
    }

    pub fn add(&mut self, task: FurTask) {
        self.total_time += (task.stop_time - task.start_time).num_seconds();
        self.tasks.push(task);
    }

    pub fn is_equal_to(&self, task: &FurTask) -> bool {
        if self.name == task.name
            && self.tags == task.tags
            && self.project.to_lowercase() == task.project.to_lowercase()
            && self.rate == task.rate
        {
            true
        } else {
            false
        }
    }

    pub fn all_task_ids(&self) -> Vec<u32> {
        self.tasks.iter().map(|task| task.id).collect()
    }
}

impl fmt::Display for FurTaskGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if !self.project.is_empty() {
            write!(f, " @{}", self.project)?;
        }

        if !self.tags.is_empty() {
            write!(f, " #{}", self.tags)?;
        }

        if self.rate != 0.0 {
            write!(f, " ${:.2}", self.rate)?;
        }

        Ok(())
    }
}
