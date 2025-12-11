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

use super::fur_shortcut::FurShortcut;

#[derive(Clone, Debug)]
pub struct ShortcutToEdit {
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
    pub uid: String,
    pub invalid_input_error_message: String,
}

impl ShortcutToEdit {
    pub fn new_from(shortcut: &FurShortcut) -> Self {
        let color = Color::from_rgb8(
            u8::from_str_radix(&shortcut.color_hex.get(1..3).unwrap_or("b1"), 16).unwrap_or(177),
            u8::from_str_radix(&shortcut.color_hex.get(3..5).unwrap_or("79"), 16).unwrap_or(121),
            u8::from_str_radix(&shortcut.color_hex.get(5..7).unwrap_or("f1"), 16).unwrap_or(241),
        );
        ShortcutToEdit {
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
            uid: shortcut.uid.clone(),
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
