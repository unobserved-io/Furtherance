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

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::{
    self, ExtIdleNotificationV1,
};
use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::ExtIdleNotifierV1;
use wayland_protocols_plasma::idle::client::org_kde_kwin_idle::OrgKdeKwinIdle;
use wayland_protocols_plasma::idle::client::org_kde_kwin_idle_timeout::{
    self, OrgKdeKwinIdleTimeout,
};

struct IdleState {
    idle_since: Option<std::time::Instant>,
}

impl IdleState {
    fn new() -> Self {
        Self { idle_since: None }
    }
}

lazy_static::lazy_static! {
    static ref IDLE_STATE: Arc<Mutex<IdleState>> = Arc::new(Mutex::new(IdleState::new()));
    static ref WAYLAND_INITIALIZED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref MONITOR_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref STOP_SIGNAL: Arc<Mutex<Option<Sender<()>>>> = Arc::new(Mutex::new(None));
}

enum IdleManager {
    Kde(OrgKdeKwinIdle),
    Standard(ExtIdleNotifierV1),
}

struct WaylandState {
    idle_state: Arc<Mutex<IdleState>>,
    seats: HashMap<u32, WlSeat>,
    idle_manager: Option<IdleManager>,
}

impl WaylandState {
    fn new(idle_state: Arc<Mutex<IdleState>>) -> Self {
        Self {
            idle_state,
            seats: HashMap::new(),
            idle_manager: None,
        }
    }

    fn handle_global(
        &mut self,
        registry: &WlRegistry,
        name: u32,
        interface: String,
        version: u32,
        qh: &QueueHandle<Self>,
    ) {
        match &interface[..] {
            "wl_seat" => {
                let seat = registry.bind::<WlSeat, _, _>(name, version, qh, ());
                if let Some(idle_manager) = &self.idle_manager {
                    let timeout_ms = 1000; // 1 second
                    match idle_manager {
                        IdleManager::Kde(manager) => {
                            let _timeout = manager.get_idle_timeout(&seat, timeout_ms, qh, ());
                        }
                        IdleManager::Standard(manager) => {
                            let _notification =
                                manager.get_idle_notification(timeout_ms, &seat, qh, name);
                        }
                    }
                }
                self.seats.insert(name, seat);
            }
            "org_kde_kwin_idle" => {
                let idle_manager: OrgKdeKwinIdle = registry.bind(name, version, qh, ());
                // Set up idle timeouts for existing seats
                for (_, seat) in &self.seats {
                    let _timeout = idle_manager.get_idle_timeout(seat, 1000, qh, ());
                }
                self.idle_manager = Some(IdleManager::Kde(idle_manager));
            }
            "ext_idle_notifier_v1" => {
                let idle_manager: ExtIdleNotifierV1 = registry.bind(name, version, qh, ());
                // Set up idle notifications for existing seats
                for (name, seat) in &self.seats {
                    let _notification = idle_manager.get_idle_notification(1000, seat, qh, *name);
                }
                self.idle_manager = Some(IdleManager::Standard(idle_manager));
            }
            _ => {}
        }
    }
}

impl Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => state.handle_global(registry, name, interface, version, qh),
            _ => {}
        }
    }
}

impl Dispatch<OrgKdeKwinIdleTimeout, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &OrgKdeKwinIdleTimeout,
        event: org_kde_kwin_idle_timeout::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            org_kde_kwin_idle_timeout::Event::Idle => {
                if let Ok(mut state) = state.idle_state.lock() {
                    state.idle_since = Some(std::time::Instant::now());
                }
            }
            org_kde_kwin_idle_timeout::Event::Resumed => {
                if let Ok(mut state) = state.idle_state.lock() {
                    state.idle_since = None;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtIdleNotificationV1, u32> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => {
                if let Ok(mut state) = state.idle_state.lock() {
                    state.idle_since = Some(std::time::Instant::now());
                }
            }
            ext_idle_notification_v1::Event::Resumed => {
                if let Ok(mut state) = state.idle_state.lock() {
                    state.idle_since = None;
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: <WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<OrgKdeKwinIdle, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &OrgKdeKwinIdle,
        _event: <OrgKdeKwinIdle as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &ExtIdleNotifierV1,
        _event: <ExtIdleNotifierV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

fn run_wayland_monitor(rx: Receiver<()>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qh, ());

    let state = WaylandState::new(IDLE_STATE.clone());
    let mut state = state;

    loop {
        if !MONITOR_RUNNING.load(Ordering::SeqCst) {
            break;
        }

        // Check if we received a stop signal
        if rx.try_recv().is_ok() {
            break;
        }

        event_queue.blocking_dispatch(&mut state)?;
        thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

pub fn initialize_wayland() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(mut initialized) = WAYLAND_INITIALIZED.lock() {
        if *initialized {
            return Ok(());
        }

        MONITOR_RUNNING.store(true, Ordering::SeqCst);

        // Create a channel for stop signaling
        let (tx, rx) = channel();
        if let Ok(mut stop_signal) = STOP_SIGNAL.lock() {
            *stop_signal = Some(tx);
        }

        thread::spawn(move || {
            if let Err(e) = run_wayland_monitor(rx) {
                eprintln!("Wayland monitor error: {}", e);
            }
        });

        *initialized = true;
    }
    Ok(())
}

pub fn get_idle_time() -> u64 {
    if !MONITOR_RUNNING.load(Ordering::SeqCst) {
        return 0;
    }

    if let Ok(state) = IDLE_STATE.lock() {
        if let Some(idle_since) = state.idle_since {
            idle_since.elapsed().as_secs()
        } else {
            0
        }
    } else {
        0
    }
}

pub fn start_idle_monitor() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Stop any existing monitor and wait for confirmation
    stop_idle_monitor();

    // Reset idle state
    if let Ok(mut state) = IDLE_STATE.lock() {
        state.idle_since = None;
    }

    if let Ok(mut initialized) = WAYLAND_INITIALIZED.lock() {
        if !*initialized {
            MONITOR_RUNNING.store(true, Ordering::SeqCst);

            // Create a channel for stop signaling
            let (tx, rx) = channel();
            if let Ok(mut stop_signal) = STOP_SIGNAL.lock() {
                *stop_signal = Some(tx);
            }

            thread::spawn(move || {
                if let Err(e) = run_wayland_monitor(rx) {
                    eprintln!("Wayland monitor error: {}", e);
                }
            });

            *initialized = true;
        }
    }
    Ok(())
}

pub fn stop_idle_monitor() {
    MONITOR_RUNNING.store(false, Ordering::SeqCst);

    // Signal the monitor thread to stop
    if let Ok(stop_signal) = STOP_SIGNAL.lock() {
        if let Some(tx) = stop_signal.as_ref() {
            let _ = tx.send(());
        }
    }

    // Reset idle state
    if let Ok(mut state) = IDLE_STATE.lock() {
        state.idle_since = None;
    }

    // Reset initialized state
    if let Ok(mut initialized) = WAYLAND_INITIALIZED.lock() {
        *initialized = false;
    }

    // Clear the stop signal
    if let Ok(mut stop_signal) = STOP_SIGNAL.lock() {
        *stop_signal = None;
    }
}
