// Furtherance - Track your time without being tracked
// Copyright (C) 2026  Ricky Kresslein <r@kressle.in>
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

use std::collections::HashMap;

use chrono::{Datelike, Days, Local, NaiveDate};
use iced_aw::date_picker::Date;
use itertools::Itertools;

use crate::database::{SortBy, SortOrder, db_retrieve_all_existing_tasks};

pub struct ExportSettings {
    pub name: bool,
    pub start_time: bool,
    pub stop_time: bool,
    pub tags: bool,
    pub project: bool,
    pub rate: bool,
    pub currency: bool,
    pub total_time: bool,
    pub total_earnings: bool,
    pub filter_by_date: bool,
    pub show_start_date_picker: bool,
    pub show_end_date_picker: bool,
    pub picked_start_date: Date,
    pub picked_end_date: Date,
    pub filter_by_project: bool,
    pub list_of_projects: Vec<String>,
    pub selected_project: Option<String>,
    pub sort_order: SortOrder,
}

impl ExportSettings {
    pub fn new() -> Self {
        let thirty_days_ago = Local::now()
            .checked_sub_days(Days::new(30))
            .unwrap_or(Local::now());
        Self {
            name: true,
            start_time: true,
            stop_time: true,
            tags: true,
            project: true,
            rate: true,
            currency: true,
            total_time: true,
            total_earnings: true,
            filter_by_date: false,
            show_start_date_picker: false,
            show_end_date_picker: false,
            picked_start_date: Date::from_ymd(
                thirty_days_ago.year(),
                thirty_days_ago.month(),
                thirty_days_ago.day(),
            ),
            picked_end_date: Date::today(),
            filter_by_project: false,
            list_of_projects: Vec::new(),
            selected_project: None,
            sort_order: SortOrder::Descending,
        }
    }

    pub fn get_all_projects(&mut self) {
        let tasks_by_project =
            match db_retrieve_all_existing_tasks(SortBy::StopTime, SortOrder::Descending) {
                Ok(all_tasks) => all_tasks
                    .into_iter()
                    .into_group_map_by(|t| t.project.clone()),
                Err(e) => {
                    eprintln!("Could not fetch tasks: {e}");
                    HashMap::new()
                }
            };

        self.list_of_projects = tasks_by_project.keys().cloned().collect();
    }

    pub fn set_picked_end_date(&mut self, new_date: Date) {
        if let Some(new_end_date) =
            NaiveDate::from_ymd_opt(new_date.year, new_date.month, new_date.day)
            && let Some(start_date) = NaiveDate::from_ymd_opt(
                self.picked_start_date.year,
                self.picked_start_date.month,
                self.picked_start_date.day,
            )
        {
            if new_end_date >= start_date {
                self.picked_end_date = new_date;
                self.show_end_date_picker = false;
            }
        }
    }

    pub fn set_picked_start_date(&mut self, new_date: Date) {
        if let Some(new_start_date) =
            NaiveDate::from_ymd_opt(new_date.year, new_date.month, new_date.day)
            && let Some(end_date) = NaiveDate::from_ymd_opt(
                self.picked_end_date.year,
                self.picked_end_date.month,
                self.picked_end_date.day,
            )
        {
            if new_start_date <= end_date {
                self.picked_start_date = new_date;
                self.show_start_date_picker = false;
            }
        }
    }
}
