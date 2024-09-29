use std::env;
use std::fs;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").unwrap();

    let out_dir = env::current_dir().unwrap();
    let dest_path = out_dir.join("VERSION");

    fs::write(&dest_path, version).unwrap();

    println!("cargo:rerun-if-changed=Cargo.toml");
}
