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
use std::sync::Arc;
#[cfg(target_os = "linux")]
use tokio::runtime::Runtime;
#[cfg(target_os = "linux")]
use zbus::{proxy, Connection};

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
#[proxy(
    interface = "org.gnome.Mutter.IdleMonitor",
    default_service = "org.gnome.Mutter.IdleMonitor",
    default_path = "/org/gnome/Mutter/IdleMonitor/Core"
)]
trait GnomeIdleMonitor {
    async fn get_idletime(&self) -> zbus::Result<u64>;
}

#[cfg(target_os = "linux")]
#[proxy(
    interface = "org.kde.KIdleTime",
    default_service = "org.kde.KIdleTime",
    default_path = "/org/kde/KIdleTime"
)]
trait KdeIdleTime {
    #[zbus(name = "idleTime")]
    async fn idle_time(&self) -> zbus::Result<u64>;
}

#[cfg(target_os = "linux")]
#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/org/freedesktop/ScreenSaver"
)]
trait FreeDesktopIdleMonitor {
    async fn get_session_idle_time(&self) -> zbus::Result<u32>;
}

#[cfg(target_os = "linux")]
fn get_wayland_idle_sync() -> Result<u64, Box<dyn std::error::Error>> {
    let rt = Arc::new(Runtime::new()?);
    rt.block_on(get_wayland_idle_seconds())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[cfg(target_os = "linux")]
async fn get_wayland_idle_seconds() -> zbus::Result<u64> {
    let connection = Connection::session().await?;

    // Try GNOME Mutter IdleMonitor
    if let Ok(proxy) = GnomeIdleMonitorProxy::new(&connection).await {
        if let Ok(idle_time) = proxy.get_idletime().await {
            println!("{}", idle_time / 1000);
            return Ok(idle_time / 1000);
        }
    }

    // Try KDE IdleTime
    if let Ok(proxy) = KdeIdleTimeProxy::new(&connection).await {
        if let Ok(idle_time) = proxy.idle_time().await {
            println!("{}", idle_time / 1000);
            return Ok(idle_time / 1000);
        }
    }

    // Try other desktops
    if let Ok(proxy) = FreeDesktopIdleMonitorProxy::new(&connection).await {
        if let Ok(idle_time) = proxy.get_session_idle_time().await {
            println!("{}", idle_time);
            return Ok(idle_time as u64);
        }
    }

    // If all methods fail, return an error
    Err(zbus::Error::InvalidField)
    // let connection = Connection::session().await?;

    // let proxy = zbus::Proxy::new(
    //     &connection,
    //     "org.gnome.Mutter.IdleMonitor",
    //     "/org/gnome/Mutter/IdleMonitor/Core",
    //     "org.gnome.Mutter.IdleMonitor",
    // ).await?;

    // let idle_time: u32 = proxy.call("GetSessionIdleTime", &()).await?;

    // println!("System has been idle for {} seconds", idle_time);

    // Ok(idle_time as u64)
}

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
        match get_wayland_idle_sync() {
            Ok(seconds) => seconds,
            Err(e) => {
                println!("Error: {}", e);
                0
            }
        }
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
