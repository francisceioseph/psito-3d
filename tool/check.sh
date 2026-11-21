#!/usr/bin/env bash
# Formats, lints and tests the whole workspace, as the CI does.
#
#   tool/check.sh           everything
#   tool/check.sh --quick   skips the GPU and end-to-end tests
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
if [[ "${1:-}" == "--quick" ]]; then
  cargo test --workspace --lib --bins
  cargo test -p psito3d-render --test shaders
else
  cargo test --workspace
fi
