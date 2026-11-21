#!/bin/bash
# Writes a new version into the workspace and the AppStream release list.
# semantic-release calls it in the CI.
# Usage: tool/set-version.sh <version>
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="${1:?usage: tool/set-version.sh <version>}"
sed -i "/^\[workspace.package\]/,/^\[/s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
# The newest release goes first in the metainfo.
sed -i "s|  <releases>|  <releases>\n    <release version=\"$VERSION\" date=\"$(date -u +%F)\"/>|" packaging/org.psito.psito3d.metainfo.xml
cargo update --workspace --offline >/dev/null 2>&1 || cargo update --workspace >/dev/null
