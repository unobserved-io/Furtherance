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

#[cfg(target_os = "linux")]
mod linux_idle {
    use std::path::Path;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use users::get_current_uid;
    use zbus::{proxy, Connection};

    fn is_wayland() -> bool {
        if let Ok(_) = env::var("XDG_SESSION_TYPE").map(|v| v == "wayland") {
            return true;
        } else if let Ok(_) = env::var("WAYLAND_DISPLAY") {
            return Path::new(&format!("/run/user/{}/wayland-0", get_current_uid())).exists();
        }
        false
    }

    fn is_x11() -> bool {
        x11rb::connect(None).is_ok()
    }

    #[proxy(
        interface = "org.gnome.Mutter.IdleMonitor",
        default_service = "org.gnome.Mutter.IdleMonitor",
        default_path = "/org/gnome/Mutter/IdleMonitor/Core"
    )]
    trait GnomeIdleMonitor {
        async fn get_idletime(&self) -> zbus::Result<u64>;
    }

    fn get_wayland_idle_sync() -> Result<u64, Box<dyn std::error::Error>> {
        let rt = Arc::new(Runtime::new()?);
        rt.block_on(get_wayland_idle_seconds())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    async fn get_wayland_idle_seconds() -> zbus::Result<u64> {
        let connection = Connection::session().await?;

        // Try GNOME Mutter IdleMonitor
        if let Ok(proxy) = GnomeIdleMonitorProxy::new(&connection).await {
            if let Ok(idle_time) = proxy.get_idletime().await {
                println!("{}", idle_time / 1000);
                return Ok(idle_time / 1000);
            }
        }

        Err(zbus::Error::InvalidField)
    }

    pub fn get_linux_idle_seconds() -> u64 {
        if is_wayland() {
            match get_wayland_idle_sync() {
                Ok(seconds) => seconds,
                Err(_) => 0,
            }
        } else if is_x11() {
            get_mac_windows_x11_idle_seconds()
        } else {
            0
        }
    }
}
