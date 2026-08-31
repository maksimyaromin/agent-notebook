#!/usr/bin/env bash
# The local gate: format, lint, test.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings

if command -v cargo-nextest >/dev/null 2>&1; then
  cargo nextest run --workspace --locked
else
  cargo test --workspace --locked
fi
# nextest skips doctests, so they run on their own either way.
cargo test --workspace --locked --doc
# The Core is a library a host embeds, so a broken link in its API docs is a
# defect like any other.
# `--document-private-items` because a link is as broken for the next
# reader of this repository as for a host reading the published API.
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --document-private-items --quiet
