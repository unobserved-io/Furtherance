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

use std::env;

use user_idle::UserIdle;

#[cfg(target_os = "linux")]
pub use linux_idle::get_linux_idle_seconds;

fn get_mac_windows_x11_idle_seconds() -> u64 {
    if let Ok(idle) = UserIdle::get_time() {
        idle.as_seconds()
    } else {
        0
    }
}

pub fn get_idle_time() -> u64 {
    match env::consts::OS {
        "windows" => get_mac_windows_x11_idle_seconds(),
        "macos" => get_mac_windows_x11_idle_seconds(),
        #[cfg(target_os = "linux")]
        "linux" => get_linux_idle_seconds(),
        _ => 0,
    }
}
