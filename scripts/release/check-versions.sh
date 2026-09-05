#!/bin/sh
# One version everywhere: the Cargo workspace, every npm package, the pins
# the shim package puts on its platform packages, and the tag when given.
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
if [ -n "${1:-}" ]; then
  tagged=${1#v}
  if [ "$tagged" != "$cargo_version" ]; then
    echo "the tag names $tagged and Cargo.toml $cargo_version"
    status=1
  fi
fi
[ $status -eq 0 ] && echo "ok: every manifest is at $cargo_version${1:+, as is the tag}"
exit $status
