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

use iced::Color;

use crate::helpers::color_utils::RandomColor;

#[derive(Clone, Debug)]
pub struct ShortcutToAdd {
    pub name: String,
    pub tags: String,
    pub project: String,
    pub new_rate: String,
    pub color: Color,
    pub show_color_picker: bool,
    pub invalid_input_error_message: String,
}

impl ShortcutToAdd {
    pub fn new() -> Self {
        ShortcutToAdd {
            name: String::new(),
            tags: String::new(),
            project: String::new(),
            new_rate: format!("{:.2}", 0.0),
            color: Color::random(),
            show_color_picker: false,
            invalid_input_error_message: String::new(),
        }
    }

    pub fn input_error(&mut self, message: String) {
        self.invalid_input_error_message = message;
    }
}
