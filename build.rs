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

use std::env;
use std::fs;

#[cfg(target_os = "windows")]
use winresource::WindowsResource;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let out_dir = env::current_dir().unwrap();
    let dest_path = out_dir.join("VERSION");

    fs::write(&dest_path, version).unwrap();

    println!("cargo:rerun-if-changed=Cargo.toml");

    #[cfg(target_os = "windows")]
    WindowsResource::new()
        .set_icon("assets/windows/256x256.ico")
        .compile()
        .unwrap();
}
