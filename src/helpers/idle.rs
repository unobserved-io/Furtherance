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

use std::{env, path::Path};
use user_idle::UserIdle;
use users::get_current_uid;

#[cfg(target_os = "linux")]
use wayrs_client::{connection::Connection, global::GlobalsExt, protocol::WlSeat, IoMode};
#[cfg(target_os = "linux")]
use wayrs_protocols::ext_idle_notify_v1::{
    ext_idle_notification_v1, ExtIdleNotificationV1, ExtIdleNotifierV1,
};
#[cfg(target_os = "linux")]
use wayrs_utils::seats::{SeatHandler, Seats};
#[cfg(target_os = "linux")]
use x11rb;

fn is_wayland() -> bool {
    if let Ok(_) = env::var("XDG_SESSION_TYPE").map(|v| v == "wayland") {
        return true;
    } else if let Ok(_) = env::var("WAYLAND_DISPLAY") {
        return Path::new(&format!("/run/user/{}/wayland-0", get_current_uid())).exists();
    }
    false
}

#[cfg(target_os = "linux")]
fn is_x11() -> bool {
    x11rb::connect(None).is_ok()
}

#[cfg(target_os = "linux")]
fn get_wayland_idle_seconds() -> u64 {}

fn get_mac_windows_x11_idle_seconds() -> u64 {
    if let Ok(idle) = UserIdle::get_time() {
        idle.as_seconds()
    } else {
        0
    }
}

#[cfg(target_os = "linux")]
fn get_linux_idle_seconds() -> u64 {
    if is_wayland() {
        get_wayland_idle_seconds()
    } else if is_x11() {
        get_mac_windows_x11_idle_seconds()
    } else {
        0
    }
}

#[cfg(not(target_os = "linux"))]
fn get_linux_idle_seconds() -> u64 {
    // Fallback for non-Linux platforms
    0
}

fn get_idle_time() -> u64 {
    match env::consts::OS {
        "windows" => get_mac_windows_x11_idle_seconds(),
        "macos" => get_mac_windows_x11_idle_seconds(),
        "linux" => get_linux_idle_seconds(),
        _ => 0,
    }
}

pub fn is_idle() -> bool {
    let time_idle = get_idle_time();
    // TODO: Check if time_idle is greater than idle in settings
    false
}
