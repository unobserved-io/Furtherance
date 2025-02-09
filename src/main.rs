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

#![windows_subsystem = "windows"]

pub mod app;
mod autosave;
mod charts {
    pub mod all_charts;
    pub mod average_earnings_chart;
    pub mod average_time_chart;
    pub mod earnings_chart;
    pub mod selection_earnings_recorded_chart;
    pub mod selection_time_recorded_chart;
    pub mod time_recorded_chart;
}
mod constants;
mod database;
mod helpers {
    pub mod color_utils;
    pub mod idle;
    pub mod midnight_subscription;
    #[cfg(target_os = "linux")]
    pub mod wayland_idle;
}
mod localization;
mod models {
    pub mod fur_idle;
    pub mod fur_pomodoro;
    pub mod fur_report;
    pub mod fur_settings;
    pub mod fur_shortcut;
    pub mod fur_task;
    pub mod fur_task_group;
    pub mod fur_todo;
    pub mod fur_user;
    pub mod group_to_edit;
    pub mod shortcut_to_add;
    pub mod shortcut_to_edit;
    pub mod task_to_add;
    pub mod task_to_edit;
}
pub mod server {
    pub mod encryption;
    pub mod login;
    pub mod logout;
    pub mod sync;
}
mod style;
mod tests {
    mod timer_tests;
}
pub mod ui {
    pub mod todos;
}
mod view_enums;

use std::borrow::Cow;

use app::Furtherance;
use iced::advanced::graphics::image::image_rs::ImageFormat;

fn main() -> iced::Result {
    let window_icon = iced::window::icon::from_file_data(
        include_bytes!("../assets/icon/32x32@2x.png"),
        Some(ImageFormat::Png),
    );
    let window_settings = iced::window::Settings {
        size: iced::Size {
            width: 1024.0,
            height: 600.0,
        },
        icon: window_icon.ok(),
        ..Default::default()
    };

    let settings = iced::Settings {
        id: Some(String::from("io.unobserved.furtherance")),
        fonts: vec![
            Cow::Borrowed(iced_fonts::REQUIRED_FONT_BYTES),
            Cow::Borrowed(iced_fonts::BOOTSTRAP_FONT_BYTES),
        ],
        ..Default::default()
    };

    iced::application(Furtherance::title, Furtherance::update, Furtherance::view)
        .subscription(Furtherance::subscription)
        .theme(Furtherance::theme)
        .window(window_settings)
        .settings(settings)
        .run_with(Furtherance::new)
}
