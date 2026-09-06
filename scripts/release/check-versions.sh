#!/bin/sh
# One version everywhere: the Cargo workspace, every npm package and the pins
# the shim package puts on its platform packages.
set -eu
cd "$(dirname "$0")/../.."
cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
status=0
for manifest in packages/*/package.json; do
  declared=$(node -p "require('./$manifest').version")
  if [ "$declared" != "$cargo_version" ]; then
    echo "$manifest is at $declared and Cargo.toml at $cargo_version"
    status=1
  fi
done
pins=$(node -p "Object.values(require('./packages/agent-notebook/package.json').optionalDependencies).join(' ')")
for pin in $pins; do
  if [ "$pin" != "$cargo_version" ]; then
    echo "packages/agent-notebook/package.json pins a platform package at $pin, not $cargo_version"
    status=1
  fi
done
[ $status -eq 0 ] && echo "ok: every manifest is at $cargo_version"
exit $status
