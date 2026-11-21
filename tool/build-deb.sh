#!/bin/bash
# Builds and assembles the .deb, without sudo.
#
# The binary links only to libc; Vulkan, X11, Wayland and xkbcommon are
# opened at run time, so ldd does not show them and the dependencies are
# listed by hand. It is built on Ubuntu 24.04 (glibc 2.39): Debian 13,
# Ubuntu 24.04, Mint 22 or newer.
# Usage: tool/build-deb.sh [output folder, default target/packages] [--no-build]
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="$(realpath -m "${1:-target/packages}")"
VERSION="$(tool/version.sh)"
NAME=psito-3d

[ "${2:-}" = "--no-build" ] || cargo build --release --locked -p psito-3d

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
# THIRD_PARTY_LICENSES points at a file made before, to skip cargo-about.
if [ -n "${THIRD_PARTY_LICENSES:-}" ]; then
    cp "$THIRD_PARTY_LICENSES" "$WORK/THIRD_PARTY_LICENSES.md"
else
    tool/third-party-licenses.sh "$WORK/THIRD_PARTY_LICENSES.md"
fi
ROOT="$WORK/root"
tool/stage-package.sh "$ROOT" "$WORK/THIRD_PARTY_LICENSES.md"

mkdir -p "$ROOT/DEBIAN"
cat > "$ROOT/DEBIAN/control" <<CONTROL
Package: $NAME
Version: $VERSION
Section: graphics
Priority: optional
Architecture: amd64
Maintainer: Francisco Souza <francisceioseph@users.noreply.github.com>
Homepage: https://francisceioseph.github.io/psito-3d/
Installed-Size: $(du -sk "$ROOT/usr" | cut -f1)
Depends: libc6 (>= 2.39), libgcc-s1, libvulkan1, libxkbcommon0, libxkbcommon-x11-0, libwayland-client0, libwayland-cursor0, libx11-6, libx11-xcb1, libxcursor1, libxi6, libxrandr2, libegl1
Recommends: mesa-vulkan-drivers, xdg-desktop-portal
Description: Make landscapes of mountains, sea, sky and chrome balls
 Drag the sun across the sky, roll a new world with a die, melt chrome
 balls into each other and watch the picture sharpen while you look at it.
 Terrains and themes can come from plugins written in Lua.
 No ads, no telemetry and no account.
CONTROL

mkdir -p "$OUT"
DEB="$OUT/${NAME}_${VERSION}_amd64.deb"
dpkg-deb --root-owner-group -Zxz --build "$ROOT" "$DEB" >/dev/null
echo "$DEB"
