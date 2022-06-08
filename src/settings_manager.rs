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

use crate::config;
use gtk::{gio, gio::prelude::*, glib};

pub fn get_settings() -> gio::Settings {
    let app_id = config::APP_ID.trim_end_matches(".Devel");
    gio::Settings::new(app_id)
}

pub fn bind_property<P: IsA<glib::Object>>(key: &str, object: &P, property: &str) {
    let settings = get_settings();
    settings
        .bind(key, object, property)
        .flags(gio::SettingsBindFlags::DEFAULT)
        .build();
}

#[allow(dead_code)]
pub fn get_bool(key: &str) -> bool {
    let settings = get_settings();
    settings.boolean(key)
}

#[allow(dead_code)]
pub fn get_int(key: &str) -> i32 {
    let settings = get_settings();
    settings.int(key)
}
