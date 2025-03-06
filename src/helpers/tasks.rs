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

use std::collections::BTreeMap;

use crate::{
    database::{db_retrieve_tasks_with_day_limit, SortBy, SortOrder},
    models::{fur_task::FurTask, fur_task_group::FurTaskGroup},
};

pub fn get_task_history(limit: i64) -> BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> {
    let mut grouped_tasks_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> = BTreeMap::new();

    match db_retrieve_tasks_with_day_limit(limit, SortBy::StopTime, SortOrder::Descending) {
        Ok(all_tasks) => {
            let tasks_by_date = group_tasks_by_date(all_tasks);

            for (date, tasks) in tasks_by_date {
                let mut all_groups: Vec<FurTaskGroup> = vec![];
                for task in tasks {
                    if let Some(matching_group) =
                        all_groups.iter_mut().find(|x| x.is_equal_to(&task))
                    {
                        matching_group.add(task);
                    } else {
                        all_groups.push(FurTaskGroup::new_from(task));
                    }
                }
                grouped_tasks_by_date.insert(date, all_groups);
            }
        }
        Err(e) => {
            eprintln!("Error retrieving tasks from database: {}", e);
        }
    }
    grouped_tasks_by_date
}

fn group_tasks_by_date(tasks: Vec<FurTask>) -> BTreeMap<chrono::NaiveDate, Vec<FurTask>> {
    let mut grouped_tasks: BTreeMap<chrono::NaiveDate, Vec<FurTask>> = BTreeMap::new();

    for task in tasks {
        let date = task.start_time.date_naive(); // Extract the date part
        grouped_tasks
            .entry(date)
            .or_insert_with(Vec::new)
            .push(task);
    }

    grouped_tasks
}
