#!/bin/bash
# Writes the licenses of every crate compiled into Psito 3D, with their full
# texts, for the packages to ship. Needs cargo-about
# (cargo install --locked cargo-about).
# Usage: tool/third-party-licenses.sh <output file>
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="${1:?usage: tool/third-party-licenses.sh <output file>}"
cargo about generate --workspace --fail -o "$OUT" about.hbs
