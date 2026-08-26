#!/usr/bin/env bash
# The local gate: format, lint, test.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings

if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --workspace
else
  cargo test --workspace
fi
# nextest skips doctests, so they run on their own either way.
cargo test --workspace --doc
