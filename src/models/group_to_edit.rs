// Furtherance - Track your time without being tracked
// Copyright (C) 2025  Ricky Kresslein <rk@unobserved.io>
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

use crate::models::{fur_task::FurTask, fur_task_group::FurTaskGroup};

#[derive(Debug, Clone)]
pub struct GroupToEdit {
    pub uid: String,
    pub name: String,
    pub new_name: String,
    pub tags: String,
    pub new_tags: String,
    pub project: String,
    pub new_project: String,
    pub rate: f32,
    pub new_rate: String,
    pub tasks: Vec<FurTask>,
    pub is_in_edit_mode: bool,
    pub invalid_input_error_message: String,
}

impl GroupToEdit {
    pub fn new_from(group: &FurTaskGroup) -> Self {
        GroupToEdit {
            uid: group.uid.clone(),
            name: group.name.clone(),
            new_name: group.name.clone(),
            tags: group.tags.clone(),
            new_tags: if group.tags.is_empty() {
                group.tags.clone()
            } else {
                format!("#{}", group.tags)
            },
            project: group.project.clone(),
            new_project: group.project.clone(),
            rate: group.rate,
            new_rate: format!("{:.2}", group.rate),
            tasks: group.tasks.clone(),
            is_in_edit_mode: false,
            invalid_input_error_message: String::new(),
        }
    }

    pub fn is_changed(&self) -> bool {
        if self.name != self.new_name.trim()
            || self.tags
                != self
                    .new_tags
                    .trim()
                    .strip_prefix('#')
                    .unwrap_or(&self.tags)
                    .trim()
            || self.project != self.new_project.trim()
            || self.rate != self.new_rate.trim().parse::<f32>().unwrap_or(0.0)
        {
            true
        } else {
            false
        }
    }

    pub fn input_error(&mut self, message: String) {
        self.invalid_input_error_message = message;
    }

    pub fn all_task_ids(&self) -> Vec<String> {
        self.tasks.iter().map(|x| x.uid.clone()).collect()
    }
}
