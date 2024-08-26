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

use chrono::{DateTime, Duration, Local, NaiveTime};
use iced_aw::{date_picker::Date, time_picker::Time};

use super::group_to_edit::GroupToEdit;

#[derive(Clone, Debug)]
pub struct TaskToAdd {
    pub name: String,
    pub start_time: DateTime<Local>,
    pub displayed_start_time: Time,
    pub displayed_start_date: Date,
    pub show_start_time_picker: bool,
    pub show_start_date_picker: bool,
    pub stop_time: DateTime<Local>,
    pub displayed_stop_time: Time,
    pub displayed_stop_date: Date,
    pub show_stop_time_picker: bool,
    pub show_stop_date_picker: bool,
    pub tags: String,
    pub project: String,
    pub rate: f32,
    pub new_rate: String,
    pub invalid_input_error_message: String,
}

impl TaskToAdd {
    pub fn new() -> Self {
        let now = Local::now();
        let one_hour_ago = now - Duration::hours(1);
        TaskToAdd {
            name: String::new(),
            start_time: one_hour_ago,
            displayed_start_time: Time::from(one_hour_ago.time()),
            displayed_start_date: Date::from(one_hour_ago.date_naive()),
            show_start_time_picker: false,
            show_start_date_picker: false,
            stop_time: now,
            displayed_stop_time: Time::from(now.time()),
            displayed_stop_date: Date::from(now.date_naive()),
            show_stop_time_picker: false,
            show_stop_date_picker: false,
            tags: String::new(),
            project: String::new(),
            rate: 0.0,
            new_rate: format!("{:.2}", 0.0),
            invalid_input_error_message: String::new(),
        }
    }

    pub fn new_from(group: &GroupToEdit) -> Self {
        let begin_time = NaiveTime::from_hms_opt(12, 00, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(13, 00, 0).unwrap();
        let begin_date_time = group.tasks.first().map_or_else(
            || Local::now(),
            |first_task| {
                first_task
                    .start_time
                    .with_time(begin_time)
                    .single()
                    .unwrap_or_else(|| Local::now())
            },
        );
        let end_date_time = group.tasks.first().map_or_else(
            || Local::now(),
            |first_task| {
                first_task
                    .start_time
                    .with_time(end_time)
                    .single()
                    .unwrap_or_else(|| Local::now())
            },
        );
        TaskToAdd {
            name: group.name.clone(),
            start_time: begin_date_time,
            displayed_start_time: Time::from(begin_date_time.time()),
            displayed_start_date: Date::from(begin_date_time.date_naive()),
            show_start_time_picker: false,
            show_start_date_picker: false,
            stop_time: end_date_time,
            displayed_stop_time: Time::from(end_date_time.time()),
            displayed_stop_date: Date::from(end_date_time.date_naive()),
            show_stop_time_picker: false,
            show_stop_date_picker: false,
            tags: if group.tags.is_empty() {
                group.tags.clone()
            } else {
                format!("#{}", group.tags)
            },
            project: group.project.clone(),
            rate: group.rate,
            new_rate: format!("{:.2}", group.rate),
            invalid_input_error_message: String::new(),
        }
    }

    pub fn input_error(&mut self, message: &str) {
        self.invalid_input_error_message = message.to_string();
    }
}
