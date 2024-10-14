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
