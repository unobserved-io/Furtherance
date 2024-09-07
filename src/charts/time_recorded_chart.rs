// Furtherance - Track your time without being tracked
// Copyright (C) 2022  Ricky Kresslein <rk@lakoliu.com>
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

use crate::{
    app::Message,
    constants::{CHART_COLOR, CHART_HEIGHT, MAX_X_VALUES},
    models::fur_task::FurTask,
};
use chrono::NaiveDate;
use iced::{widget::Text, Element, Length};
use plotters::prelude::*;
use plotters_backend::DrawingBackend;
use plotters_iced::{plotters_backend, Chart, ChartWidget};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct TimeRecordedChart {
    date_time: BTreeMap<NaiveDate, i64>,
}

impl TimeRecordedChart {
    pub fn new(tasks: Vec<FurTask>) -> Self {
        Self {
            date_time: time_per_day(&tasks),
        }
    }

    pub fn view(&self) -> Element<Message> {
        if self.date_time.len() <= 1 {
            Text::new("Not enough data to show charts.").into()
        } else {
            let chart = ChartWidget::new(self)
                .width(Length::Fill)
                .height(Length::Fixed(CHART_HEIGHT));

            chart.into()
        }
    }
}

impl Chart<Message> for TimeRecordedChart {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut chart: ChartBuilder<DB>) {
        let min_time = self.date_time.values().copied().min().unwrap_or(0);
        let min_minus_five_percent = min_time as f32 - (min_time as f32 * 0.05);
        let max_time = self.date_time.values().copied().max().unwrap_or(0);

        if self.date_time.len() > 1 {
            if let Some(first_date) = self.date_time.first_key_value() {
                if let Some(last_date) = self.date_time.last_key_value() {
                    let mut chart = chart
                        .margin(30)
                        .caption(
                            "Time Recorded",
                            ("sans-serif", 15).into_font().color(&light_dark_color()),
                        )
                        .x_label_area_size(30)
                        .y_label_area_size(30)
                        .build_cartesian_2d(
                            *first_date.0..*last_date.0,
                            min_minus_five_percent as i64..max_time,
                        )
                        .unwrap();

                    chart
                        .configure_mesh()
                        .label_style(&light_dark_color())
                        .x_label_style(("sans-serif", 12).into_font().color(&light_dark_color()))
                        .x_labels(MAX_X_VALUES)
                        .y_label_style(
                            ("sans-serif", 12)
                                .into_font()
                                .color(&light_dark_color())
                                .transform(FontTransform::Rotate90),
                        )
                        .y_label_formatter(&|y| seconds_to_hms(y))
                        .axis_style(ShapeStyle::from(light_dark_color()).stroke_width(1))
                        .draw()
                        .unwrap();

                    chart
                        .draw_series(LineSeries::new(
                            self.date_time.iter().map(|(d, t)| (*d, *t)),
                            CHART_COLOR.filled(),
                        ))
                        .unwrap();
                }
            }
        }
    }
}

fn time_per_day(tasks: &Vec<FurTask>) -> BTreeMap<NaiveDate, i64> {
    let mut time_by_day = BTreeMap::new();
    for task in tasks {
        *time_by_day.entry(task.start_time.date_naive()).or_insert(0) +=
            task.total_time_in_seconds();
    }
    time_by_day
}

fn light_dark_color() -> RGBColor {
    match dark_light::detect() {
        dark_light::Mode::Light | dark_light::Mode::Default => BLACK,
        dark_light::Mode::Dark => WHITE,
    }
}

fn seconds_to_hms(total_seconds: &i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    let s = total_seconds % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}
