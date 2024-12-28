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
use {
    std::path::Path, std::sync::Arc, tokio::runtime::Runtime, uzers::get_current_uid, zbus::proxy,
};

pub fn get_mac_windows_x11_idle_seconds() -> u64 {
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
        "linux" => {
            if is_wayland() {
                if is_kde() {
                    match get_wayland_idle_sync() {
                        Ok(seconds) => seconds,
                        Err(_) => 0,
                    }
                } else if is_gnome() {
                    match get_gnome_idle_sync() {
                        Ok(seconds) => seconds,
                        Err(_) => 0,
                    }
                } else {
                    0
                }
            } else if is_x11() {
                get_mac_windows_x11_idle_seconds()
            } else {
                0
            }
        }
        _ => 0,
    }
}

#[cfg(target_os = "linux")]
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
fn get_wayland_idle_sync() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rt = Arc::new(Runtime::new()?);
    rt.block_on(get_wayland_idle_seconds())
}

#[cfg(target_os = "linux")]
async fn get_wayland_idle_seconds() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    use crate::helpers::wayland_idle;

    wayland_idle::initialize_wayland()?;

    Ok(wayland_idle::get_idle_time())
}

#[cfg(target_os = "linux")]
fn get_gnome_idle_sync() -> Result<u64, Box<dyn std::error::Error>> {
    let rt = Arc::new(Runtime::new()?);
    rt.block_on(get_gnome_idle_seconds())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

#[cfg(target_os = "linux")]
async fn get_gnome_idle_seconds() -> zbus::Result<u64> {
    let connection = zbus::Connection::session().await?;

    // Try GNOME Mutter IdleMonitor
    if let Ok(proxy) = GnomeIdleMonitorProxy::new(&connection).await {
        if let Ok(idle_time) = proxy.get_idletime().await {
            return Ok(idle_time / 1000);
        }
    }

    Err(zbus::Error::InvalidField)
}

#[cfg(target_os = "linux")]
pub fn is_kde() -> bool {
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        return desktop.to_uppercase().contains("KDE");
    }
    false
}

#[cfg(target_os = "linux")]
fn is_gnome() -> bool {
    if let Ok(xdg_current_desktop) = env::var("XDG_CURRENT_DESKTOP") {
        if xdg_current_desktop.to_lowercase().contains("gnome") {
            return true;
        }
    }

    if let Ok(gdm_session) = env::var("GDMSESSION") {
        if gdm_session.to_lowercase().contains("gnome") {
            return true;
        }
    }

    false
}
