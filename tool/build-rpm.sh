#!/bin/bash
# Builds and assembles the .rpm for Fedora, without sudo. The dependencies
# are listed by hand (AutoReq off), because the libraries the program needs
# are opened at run time and rpm cannot see them.
# Usage: tool/build-rpm.sh [output folder, default target/packages] [--no-build]
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
tool/stage-package.sh "$WORK/stage" "$WORK/THIRD_PARTY_LICENSES.md"

cat > "$WORK/$NAME.spec" <<SPEC
%global debug_package %{nil}
%global __os_install_post %{nil}
%global _build_id_links none

Name: $NAME
Version: $VERSION
Release: 1
Summary: Make landscapes of mountains, sea, sky and chrome balls
License: Apache-2.0
URL: https://francisceioseph.github.io/psito-3d/
AutoReqProv: no
Requires: glibc >= 2.39
Requires: vulkan-loader
Requires: libxkbcommon
Requires: libxkbcommon-x11
Requires: libwayland-client
Requires: libwayland-cursor
Requires: libX11
Requires: libX11-xcb
Requires: libXcursor
Requires: libXi
Requires: libXrandr
Requires: libglvnd-egl
Recommends: mesa-vulkan-drivers
Recommends: xdg-desktop-portal

%description
Drag the sun across the sky, roll a new world with a die, melt chrome balls
into each other and watch the picture sharpen while you look at it. Terrains
and themes can come from plugins written in Lua.
No ads, no telemetry and no account.

%install
cp -a "$WORK/stage/." %{buildroot}/

%files
/usr/bin/psito-3d
/usr/share/$NAME
/usr/share/applications/org.psito.psito3d.desktop
/usr/share/metainfo/org.psito.psito3d.metainfo.xml
/usr/share/icons/hicolor/*/apps/org.psito.psito3d.png
/usr/share/doc/$NAME
SPEC

mkdir -p "$OUT"
rpmbuild -bb "$WORK/$NAME.spec" --target x86_64 \
    --define "_topdir $WORK/rpmbuild" \
    --define "_rpmdir $OUT" \
    --define "_build_name_fmt %%{NAME}-%%{VERSION}-%%{RELEASE}.%%{ARCH}.rpm" \
    >/dev/null 2>"$WORK/rpmbuild.log" || { cat "$WORK/rpmbuild.log" >&2; exit 1; }
echo "$OUT/$NAME-$VERSION-1.x86_64.rpm"
