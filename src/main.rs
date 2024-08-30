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

pub mod app;
mod constants;
mod database;
mod helpers {
    pub mod color_utils;
    pub mod idle;
}
mod models {
    pub mod fur_idle;
    pub mod fur_pomodoro;
    pub mod fur_settings;
    pub mod fur_shortcut;
    pub mod fur_task;
    pub mod fur_task_group;
    pub mod group_to_edit;
    pub mod shortcut_to_add;
    pub mod task_to_add;
    pub mod task_to_edit;
}
mod style;
mod tests {
    mod timer_tests;
}
mod view_enums;

use app::Furtherance;
use iced::{multi_window::Application, window, Settings, Size};

fn main() -> iced::Result {
    Furtherance::run(Settings {
        window: window::Settings {
            size: Size {
                height: 600.0,
                width: 1024.0,
            },
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
