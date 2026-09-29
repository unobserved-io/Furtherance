// Furtherance - Track your time without being tracked

use std::sync::OnceLock;
use zbus::blocking::Connection;
use zbus::proxy;

#[proxy(
    interface = "org.gnome.Mutter.IdleMonitor",
    default_service = "org.gnome.Mutter.IdleMonitor",
    default_path = "/org/gnome/Mutter/IdleMonitor/Core",
    gen_blocking = true
)]
trait GnomeIdleMonitor {
    fn get_idletime(&self) -> zbus::Result<u64>;
}

static DBUS_CONNECTION: OnceLock<Connection> = OnceLock::new();

pub fn get_idle_seconds() -> zbus::Result<u64> {
    let connection = match DBUS_CONNECTION.get() {
        Some(connection) => connection,
        None => {
            let connection = Connection::session()?;
            let _ = DBUS_CONNECTION.set(connection);
            DBUS_CONNECTION
                .get()
                .expect("D-Bus connection should have been initialized")
        }
    };

    let proxy = GnomeIdleMonitorProxyBlocking::new(connection)?;

    Ok(proxy.get_idletime()? / 1000)
}

pub fn is_gnome() -> bool {
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP")
        && desktop.to_lowercase().contains("gnome")
    {
        return true;
    }

    if let Ok(session) = std::env::var("GDMSESSION")
        && session.to_lowercase().contains("gnome")
    {
        return true;
    }

    false
}
