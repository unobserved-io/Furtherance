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

use iced::Color;

use crate::{
    constants::FURTHERANCE_PURPLE,
    helpers::color_utils::{FromHex, ToIcedColor},
};

use super::fur_shortcut::FurShortcut;

#[derive(Clone, Debug)]
pub struct ShortcutToEdit {
    pub id: u32,
    pub name: String,
    pub new_name: String,
    pub tags: String,
    pub new_tags: String,
    pub project: String,
    pub new_project: String,
    pub rate: f32,
    pub new_rate: String,
    pub color: Color,
    pub new_color: Color,
    pub show_color_picker: bool,
    pub invalid_input_error_message: String,
}

impl ShortcutToEdit {
    pub fn new_from(shortcut: &FurShortcut) -> Self {
        let color = Color::from_hex(shortcut.color_hex.as_str())
            .unwrap_or(FURTHERANCE_PURPLE.to_iced_color());
        ShortcutToEdit {
            id: shortcut.id,
            name: shortcut.name.clone(),
            new_name: shortcut.name.clone(),
            tags: shortcut.tags.clone(),
            new_tags: shortcut.tags.clone(),
            project: shortcut.project.clone(),
            new_project: shortcut.project.clone(),
            rate: shortcut.rate,
            new_rate: format!("{:.2}", shortcut.rate),
            color,
            new_color: color,
            show_color_picker: false,
            invalid_input_error_message: "".to_string(),
        }
    }

    pub fn is_changed(&self) -> bool {
        self.name != self.new_name
            || self.tags
                != self
                    .new_tags
                    .trim()
                    .strip_prefix('#')
                    .unwrap_or(&self.tags)
                    .trim()
            || self.project != self.new_project.trim()
            || self.rate != self.new_rate.parse::<f32>().unwrap_or(0.0)
            || self.color != self.new_color
    }

    pub fn input_error(&mut self, message: String) {
        self.invalid_input_error_message = message;
    }
}
