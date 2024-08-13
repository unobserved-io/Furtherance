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

mod app;
mod constants;
mod database;
mod fur_task;

use app::Furtherance;
use iced::{multi_window::Application, window, Settings, Size};

fn main() -> iced::Result {
    Furtherance::run(Settings {
        window: window::Settings {
            size: Size {
                height: 480.0,
                width: 400.0,
            },
            min_size: Some(Size {
                height: 480.0,
                width: 400.0,
            }),
            max_size: Some(Size {
                height: 480.0,
                width: 400.0,
            }),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
