#!/bin/bash
# Prints the version of the workspace (Cargo.toml, [workspace.package]).
set -euo pipefail
cd "$(dirname "$0")/.."
sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml
