#!/bin/bash

ARCH=${1:-amd64}
TARGET="furtherance"
VERSION=$(cat VERSION)
PROFILE="release"
RELEASE_DIR="target/$PROFILE/bundle/deb"
ARCHIVE_NAME="${TARGET}_${VERSION}_${ARCH}.deb"
ARCHIVE_PATH="$RELEASE_DIR/$ARCHIVE_NAME"

cargo install cargo-bundle
cargo bundle --$PROFILE
cp -fp $ARCHIVE_PATH "target/release/$TARGET-$VERSION-$ARCH.deb"
