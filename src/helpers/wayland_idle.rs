// Furtherance - Track your time without being tracked
// Copyright (C) 2025  Ricky Kresslein <r@kressle.in>
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

use anyhow::Context;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use wayrs_client::protocol::WlSeat;
use wayrs_client::{Connection, EventCtx, IoMode};
use wayrs_protocols::ext_idle_notify_v1::{
    ext_idle_notification_v1, ExtIdleNotificationV1, ExtIdleNotifierV1,
};
use wayrs_utils::seats::{SeatHandler, Seats};

pub struct WaylandIdleMonitor {
    is_idle: Arc<AtomicBool>,
}

impl WaylandIdleMonitor {
    pub fn spawn(timeout_secs: u64) -> Self {
        let is_idle = Arc::new(AtomicBool::new(false));
        let is_idle_clone = Arc::clone(&is_idle);

        thread::spawn(move || {
            if let Err(e) = run_monitor(timeout_secs * 1000, is_idle_clone) {
                eprintln!("Wayland idle monitor error: {:?}", e);
            }
        });

        WaylandIdleMonitor { is_idle }
    }

    pub fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::Relaxed)
    }
}

fn run_monitor(timeout_ms: u64, is_idle: Arc<AtomicBool>) -> anyhow::Result<()> {
    let mut conn = Connection::connect()?;

    let mut state = State {
        seats: Seats::new(&mut conn),
        seat_names: HashMap::default(),
        idle_state: is_idle,
    };

    conn.blocking_roundtrip()?;
    conn.dispatch_events(&mut state);
    conn.blocking_roundtrip()?;
    conn.dispatch_events(&mut state);

    let seat = state.seats.iter().next().context("No seats found")?;

    let idle_notifier = conn.bind_singleton::<ExtIdleNotifierV1>(1..=1)?;

    idle_notifier.get_idle_notification_with_cb(
        &mut conn,
        timeout_ms as u32,
        seat,
        notification_cb,
    );

    loop {
        conn.flush(IoMode::Blocking)?;
        conn.recv_events(IoMode::Blocking)?;
        conn.dispatch_events(&mut state);
    }
}

struct State {
    seats: Seats,
    seat_names: HashMap<CString, WlSeat>,
    idle_state: Arc<AtomicBool>,
}

impl SeatHandler for State {
    fn get_seats(&mut self) -> &mut Seats {
        &mut self.seats
    }
    fn seat_name(&mut self, _: &mut Connection<Self>, wl_seat: WlSeat, name: CString) {
        self.seat_names.insert(name, wl_seat);
    }
}

fn notification_cb(ctx: EventCtx<State, ExtIdleNotificationV1>) {
    match ctx.event {
        ext_idle_notification_v1::Event::Idled => {
            ctx.state.idle_state.store(true, Ordering::Relaxed);
        }
        ext_idle_notification_v1::Event::Resumed => {
            ctx.state.idle_state.store(false, Ordering::Relaxed);
        }
        _ => {}
    }
}
