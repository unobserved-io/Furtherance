#!/bin/bash
ARCH=${1:-x64}
WXS_FILE="wix/main.wxs"
FURTHERANCE_VERSION=$(cat VERSION)

# build the binary
scripts/build-windows.sh $([[ "$ARCH" == "arm64" ]] && echo "aarch64" || echo "x86_64")

# install latest wix
dotnet tool install --global wix

# add required wix extension
wix extension add -g WixToolset.UI.wixext/6.0.2

# build the installer
wix build -pdbtype none -arch $ARCH -d PackageVersion=$FURTHERANCE_VERSION $WXS_FILE -o target/release/furtherance-installer-$ARCH.msi -ext WixToolset.UI.wixext
