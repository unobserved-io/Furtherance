#!/bin/bash
EXE_NAME="furtherance.exe"
TARGET="x86_64-pc-windows-msvc"
FURTHERANCE_VERSION=$(cat VERSION).0

# update package version on Cargo.toml
cargo install cargo-edit
cargo set-version $FURTHERANCE_VERSION

# build binary
rustup target add $TARGET
cargo build --release --target=$TARGET
cp -fp target/$TARGET/release/$EXE_NAME target/release
