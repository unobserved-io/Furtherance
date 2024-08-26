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

use chrono::{DateTime, Duration, Local};

#[derive(Clone, Debug)]
pub struct FurIdle {
    pub notified: bool,
    pub reached: bool,
    pub start_time: DateTime<Local>,
}

impl FurIdle {
    pub fn new() -> Self {
        FurIdle {
            notified: false,
            reached: false,
            start_time: Local::now(),
        }
    }

    pub fn duration(&self) -> String {
        let duration = Local::now() - self.start_time;
        Self::format_duration(duration)
    }

    fn format_duration(duration: Duration) -> String {
        let std_duration =
            std::time::Duration::from_nanos(duration.num_nanoseconds().unwrap_or(0) as u64);
        let hours = std_duration.as_secs() / 3600;
        let minutes = (std_duration.as_secs() % 3600) / 60;
        let seconds = std_duration.as_secs() % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
