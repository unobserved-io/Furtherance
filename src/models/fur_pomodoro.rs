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

use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct FurPomodoro {
    pub on_break: bool,
    pub sessions: u16,
    pub snoozed: bool,
    pub snoozed_at: DateTime<Local>,
}

impl FurPomodoro {
    pub fn new() -> Self {
        FurPomodoro {
            on_break: false,
            sessions: 0,
            snoozed: false,
            snoozed_at: Local::now(),
        }
    }
}
