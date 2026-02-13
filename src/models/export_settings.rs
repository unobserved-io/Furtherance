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

use itertools::Itertools;

use crate::database::{self, db_retrieve_all_existing_tasks};

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
    pub filter_by_project: bool,
    pub list_of_projects: Vec<String>,
    pub selected_project: Option<String>,
}

impl ExportSettings {
    pub fn new() -> Self {
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
            filter_by_project: false,
            list_of_projects: Vec::new(),
            selected_project: None,
        }
    }

    pub fn get_all_projects(&mut self) {
        let tasks_by_project = match db_retrieve_all_existing_tasks(
            database::SortBy::StopTime,
            database::SortOrder::Descending,
        ) {
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
}
