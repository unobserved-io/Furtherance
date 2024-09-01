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

use std::collections::HashMap;

use chrono::{Datelike, Days, Duration, Local, NaiveDate, Utc};
use iced_aw::date_picker::Date;

use crate::{
    charts::{
        average_earnings_chart::AverageEarningsChart, average_time_chart::AverageTimeChart,
        earnings_chart::EarningsChart, time_recorded_chart::TimeRecordedChart,
    },
    database::db_retrieve_tasks_by_date_range,
    view_enums::{FurDateRange, FurTaskProperty, TabId},
};

use super::fur_task::FurTask;

#[derive(Clone, Debug)]
pub struct FurReport {
    pub active_tab: TabId,
    pub average_earnings_chart: AverageEarningsChart,
    pub average_time_chart: AverageTimeChart,
    date_range_end: NaiveDate,
    date_range_start: NaiveDate,
    pub picked_date_range: Option<FurDateRange>,
    pub picked_end_date: Date,
    pub picked_start_date: Date,
    pub picked_task_property_key: Option<FurTaskProperty>,
    pub picked_task_property_value: Option<String>,
    pub show_end_date_picker: bool,
    pub show_start_date_picker: bool,
    pub total_time: i64,
    pub total_earned: f32,
    pub tasks_in_range: Vec<FurTask>,
    pub task_property_value_keys: Vec<String>,
    pub task_property_values: HashMap<String, Vec<FurTask>>,
    pub time_recorded_chart: TimeRecordedChart,
    pub earnings_chart: EarningsChart,
}

impl FurReport {
    pub fn new() -> Self {
        let thirty_days_ago = Utc::now()
            .checked_sub_days(Days::new(30))
            .unwrap_or(Utc::now());
        let mut fur_report = FurReport {
            active_tab: TabId::Charts,
            average_earnings_chart: AverageEarningsChart::new(vec![]),
            average_time_chart: AverageTimeChart::new(vec![]),
            date_range_end: Local::now().date_naive(),
            date_range_start: (Local::now() - Duration::days(30)).date_naive(),
            earnings_chart: EarningsChart::new(vec![]),
            picked_date_range: Some(FurDateRange::ThirtyDays),
            picked_end_date: Date::today(),
            picked_start_date: Date::from_ymd(
                thirty_days_ago.year(),
                thirty_days_ago.month(),
                thirty_days_ago.day(),
            ),
            picked_task_property_key: Some(FurTaskProperty::Title),
            picked_task_property_value: None,
            show_end_date_picker: false,
            show_start_date_picker: false,
            total_time: 0,
            total_earned: 0.0,
            tasks_in_range: vec![],
            task_property_value_keys: vec![],
            task_property_values: HashMap::new(),
            time_recorded_chart: TimeRecordedChart::new(vec![]),
        };

        fur_report.update_tasks_in_range();

        fur_report
    }

    pub fn set_picked_date_ranged(&mut self, new_range: FurDateRange) {
        if self.picked_date_range != Some(new_range) {
            self.picked_date_range = Some(new_range);
            match new_range {
                FurDateRange::PastWeek => {
                    self.date_range_start = (Local::now() - Duration::days(7)).date_naive();
                    self.date_range_end = Local::now().date_naive();
                }
                FurDateRange::ThirtyDays => {
                    self.date_range_start = (Local::now() - Duration::days(30)).date_naive();
                    self.date_range_end = Local::now().date_naive();
                }
                FurDateRange::SixMonths => {
                    self.date_range_start = self.subtract_months(Local::now().date_naive(), 6);
                    self.date_range_end = Local::now().date_naive();
                }
                FurDateRange::AllTime => {
                    self.date_range_start = NaiveDate::parse_from_str("1971-01-01", "%Y-%m-%d")
                        .unwrap_or(Local::now().date_naive());
                    self.date_range_end = NaiveDate::parse_from_str("2300-01-01", "%Y-%m-%d")
                        .unwrap_or(Local::now().date_naive());
                }
                FurDateRange::Range => {
                    if let Some(new_start_date) = NaiveDate::from_ymd_opt(
                        self.picked_start_date.year,
                        self.picked_start_date.month,
                        self.picked_start_date.day,
                    ) {
                        if let Some(new_end_date) = NaiveDate::from_ymd_opt(
                            self.picked_end_date.year,
                            self.picked_end_date.month,
                            self.picked_end_date.day,
                        ) {
                            if new_start_date <= new_end_date {
                                self.date_range_start = new_start_date;
                                self.date_range_end = new_end_date;
                            }
                        }
                    }
                }
            }
            self.update_tasks_in_range();
        }
    }

    pub fn set_picked_task_property_key(&mut self, new_property: FurTaskProperty) {
        if self.picked_task_property_key != Some(new_property) {
            self.picked_task_property_key = Some(new_property);
            self.populate_task_property_values();
            self.update_selection_charts();
        }
    }

    pub fn set_picked_task_property_value(&mut self, new_value: String) {
        if self.picked_task_property_value != Some(new_value.clone()) {
            self.picked_task_property_value = Some(new_value);
            self.update_selection_charts();
        }
    }

    pub fn set_date_range_end(&mut self, new_date: Date) {
        if let Some(new_end_date) =
            NaiveDate::from_ymd_opt(new_date.year, new_date.month, new_date.day)
        {
            if self.date_range_end != new_end_date && new_end_date >= self.date_range_start {
                self.picked_end_date = new_date;
                self.date_range_end = new_end_date;
                self.show_end_date_picker = false;
                self.update_tasks_in_range();
            }
        }
    }

    pub fn set_date_range_start(&mut self, new_date: Date) {
        if let Some(new_start_date) =
            NaiveDate::from_ymd_opt(new_date.year, new_date.month, new_date.day)
        {
            if self.date_range_start != new_start_date && new_start_date <= self.date_range_end {
                self.picked_start_date = new_date;
                self.date_range_start = new_start_date;
                self.show_start_date_picker = false;
                self.update_tasks_in_range();
            }
        }
    }

    fn update_tasks_in_range(&mut self) {
        match db_retrieve_tasks_by_date_range(
            self.date_range_start.to_string(),
            self.date_range_end.to_string(),
        ) {
            Ok(s) => self.tasks_in_range = s,
            Err(e) => {
                self.tasks_in_range = vec![];
                eprintln!("Could not retrieve data in range: {}", e);
            }
        }

        self.populate_task_property_values();
        self.update_charts();
    }

    fn update_charts(&mut self) {
        (self.total_time, self.total_earned) = self.tasks_in_range.iter().fold(
            (0, 0.0),
            |(time_accumulated, earned_accumulated), task| {
                (
                    time_accumulated + task.total_time_in_seconds(),
                    earned_accumulated + task.total_earnings(),
                )
            },
        );

        self.time_recorded_chart = TimeRecordedChart::new(self.tasks_in_range.clone());
        self.earnings_chart = EarningsChart::new(self.tasks_in_range.clone());
        self.average_time_chart = AverageTimeChart::new(self.tasks_in_range.clone());
        self.average_earnings_chart = AverageEarningsChart::new(self.tasks_in_range.clone());
        self.update_selection_charts();
    }

    fn update_selection_charts(&mut self) {}

    fn subtract_months(&self, date: NaiveDate, months: i32) -> NaiveDate {
        let mut year = date.year();
        let mut month = date.month() as i32;

        month -= months;
        while month <= 0 {
            month += 12;
            year -= 1;
        }

        let day = date.day();
        NaiveDate::from_ymd_opt(year, month as u32, day)
            .or_else(|| {
                NaiveDate::from_ymd_opt(
                    year,
                    month as u32,
                    self.last_day_of_month(year, month as u32),
                )
            })
            .expect("Invalid date")
    }

    fn last_day_of_month(&self, year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if self.is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => panic!("Invalid month"),
        }
    }

    fn is_leap_year(&self, year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn populate_task_property_values(&mut self) {
        if let Some(property_key) = self.picked_task_property_key {
            self.task_property_values =
                self.tasks_in_range
                    .iter()
                    .fold(HashMap::new(), |mut accumulated, task| {
                        let keys = match property_key {
                            FurTaskProperty::Title => vec![task.name.to_string()],
                            FurTaskProperty::Project => vec![if task.project.trim().is_empty() {
                                "None".to_string()
                            } else {
                                task.project.to_string()
                            }],
                            FurTaskProperty::Tags => {
                                let tags = task
                                    .tags
                                    .split('#')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect::<Vec<String>>();
                                if tags.is_empty() {
                                    vec!["no tags".to_string()]
                                } else {
                                    tags
                                }
                            }
                            FurTaskProperty::Rate => vec![if task.rate == 0.0 {
                                "None".to_string()
                            } else {
                                format!("${:.2}", task.rate)
                            }],
                        };

                        for key in keys {
                            accumulated
                                .entry(key)
                                .or_insert_with(Vec::new)
                                .push(task.clone());
                        }
                        accumulated
                    });

            self.task_property_value_keys = self.task_property_values.keys().cloned().collect();

            // Sort keys
            match property_key {
                FurTaskProperty::Rate => {
                    self.task_property_value_keys.sort_by(|a, b| {
                        if a == "None" {
                            std::cmp::Ordering::Greater
                        } else if b == "None" {
                            std::cmp::Ordering::Less
                        } else {
                            b.cmp(a)
                        }
                    });
                }
                _ => self
                    .task_property_value_keys
                    .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase())),
            }

            if let Some(value) = self.task_property_value_keys.first() {
                self.picked_task_property_value = Some(value.to_owned());
            }
        }
    }
}
