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
use iced_aw::{date_picker::Date, time_picker::Time};

use super::fur_task::FurTask;

#[derive(Clone, Debug)]
pub struct TaskToEdit {
    pub id: u32,
    pub name: String,
    pub new_name: String,
    pub start_time: DateTime<Local>,
    pub new_start_time: DateTime<Local>,
    pub displayed_start_time: Time,
    pub displayed_start_date: Date,
    pub show_displayed_start_time_picker: bool,
    pub show_displayed_start_date_picker: bool,
    pub stop_time: DateTime<Local>,
    pub new_stop_time: DateTime<Local>,
    pub displayed_stop_time: Time,
    pub displayed_stop_date: Date,
    pub show_displayed_stop_time_picker: bool,
    pub show_displayed_stop_date_picker: bool,
    pub tags: String,
    pub new_tags: String,
    pub project: String,
    pub new_project: String,
    pub rate: f32,
    pub new_rate: String,
    pub invalid_input_error_message: String,
}

impl TaskToEdit {
    pub fn new_from(task: &FurTask) -> Self {
        TaskToEdit {
            id: task.id,
            name: task.name.clone(),
            new_name: task.name.clone(),
            start_time: task.start_time,
            new_start_time: task.start_time,
            displayed_start_time: Time::from(task.start_time.naive_local().time()),
            displayed_start_date: Date::from(task.start_time.date_naive()),
            show_displayed_start_time_picker: false,
            show_displayed_start_date_picker: false,
            stop_time: task.stop_time,
            new_stop_time: task.stop_time,
            displayed_stop_time: Time::from(task.stop_time.naive_local().time()),
            displayed_stop_date: Date::from(task.stop_time.date_naive()),
            show_displayed_stop_time_picker: false,
            show_displayed_stop_date_picker: false,
            tags: task.tags.clone(),
            new_tags: if task.tags.is_empty() {
                task.tags.clone()
            } else {
                format!("#{}", task.tags)
            },
            project: task.project.clone(),
            new_project: task.project.clone(),
            rate: task.rate,
            new_rate: format!("{:.2}", task.rate),
            invalid_input_error_message: String::new(),
        }
    }

    pub fn is_changed(&self) -> bool {
        if self.name != self.new_name.trim()
            || self.start_time != self.new_start_time
            || self.stop_time != self.new_stop_time
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
}
