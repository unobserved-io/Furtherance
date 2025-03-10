#!/bin/bash

RELEASE_DIR="target/release"
APP_DIR="$RELEASE_DIR/macos"
APP_NAME="Furtherance.app"
VERSION=$(cat VERSION)
DMG_NAME="furtherance-$VERSION.dmg"
DMG_DIR="$RELEASE_DIR/macos"

# package dmg
echo "Packing disk image..."
ln -sf /Applications "$DMG_DIR/Applications"
sleep 5
hdiutil create "$DMG_DIR/$DMG_NAME" -volname "Furtherance" -fs HFS+ -srcfolder "$APP_DIR" -ov -format UDZO
echo "Packed '$APP_NAME' in '$APP_DIR'"
