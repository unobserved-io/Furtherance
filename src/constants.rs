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

use palette::Srgb;
use plotters::{self, style::RGBColor};

pub const ALLOWED_DB_EXTENSIONS: &[&str] =
    &["db", "sqlite", "sqlite3", "db3", "database", "data", "s3db"];
pub const DEBUG_MODE: bool = cfg!(debug_assertions);
pub const FURTHERANCE_PURPLE: Srgb = Srgb::new(0.694, 0.475, 0.945);
pub const SETTINGS_SPACING: f32 = 15.0;

// Charts
pub const CHART_HEIGHT: f32 = 400.0;
pub const CHART_COLOR: RGBColor = plotters::style::colors::BLUE;
pub const MAX_X_VALUES: usize = 9;
