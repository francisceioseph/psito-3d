#!/bin/bash
# Puts into <root> the files every Linux package installs, from the release
# build. build-deb.sh and build-rpm.sh call it.
# Usage: tool/stage-package.sh <root> <third-party licenses file>
set -euo pipefail
cd "$(dirname "$0")/.."

ROOT="${1:?usage: tool/stage-package.sh <root> <licenses>}"
LICENSES="${2:?usage: tool/stage-package.sh <root> <licenses>}"
ID=org.psito.psito3d
NAME=psito-3d

install -Dm755 target/release/psito-3d "$ROOT/usr/bin/psito-3d"
# The program looks for bundled plugins in ../share/psito-3d/plugins.
mkdir -p "$ROOT/usr/share/$NAME"
cp -r plugins "$ROOT/usr/share/$NAME/"
install -Dm644 "packaging/$ID.desktop" "$ROOT/usr/share/applications/$ID.desktop"
install -Dm644 "packaging/$ID.metainfo.xml" "$ROOT/usr/share/metainfo/$ID.metainfo.xml"
for icon in assets/hicolor/*.png; do
    size="$(basename "$icon" .png)"
    install -Dm644 "$icon" "$ROOT/usr/share/icons/hicolor/$size/apps/$ID.png"
done
install -Dm644 LICENSE "$ROOT/usr/share/doc/$NAME/LICENSE"
install -Dm644 README.md "$ROOT/usr/share/doc/$NAME/README.md"
install -Dm644 "$LICENSES" "$ROOT/usr/share/doc/$NAME/THIRD_PARTY_LICENSES.md"
