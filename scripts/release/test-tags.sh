#!/bin/sh
set -eu
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' 0
mkdir -p "$fixture/scripts/release"
cp "$(dirname "$0")/check-tag.sh" "$fixture/scripts/release/check-tag.sh"
checker="$fixture/scripts/release/check-tag.sh"
printf '[workspace.package]\nversion = "0.9.0"\n' > "$fixture/Cargo.toml"

accept() {
  if ! sh "$checker" "$1" >/dev/null 2>&1; then
    echo "expected the release tag to be accepted: $1" >&2
    exit 1
  fi
}

refuse() {
  if sh "$checker" "$1" >/dev/null 2>&1; then
    echo "expected the release tag to be refused: $1" >&2
    exit 1
  fi
}

accept v2026.09.12.0.9.0
refuse v2026.09.12
refuse v2026.09.12.4
refuse v2026.09.12.0.8.0
refuse v2026.09.12.00.9.0
refuse v2026.13.12.0.9.0
refuse v2026.09.32.0.9.0
refuse v2026.09.12.0.9.0-extra
refuse ''

printf '[workspace.package]\nversion = "0.9.1"\n' > "$fixture/Cargo.toml"
accept v2026.09.12.0.9.1
refuse v2026.09.12.0.9.0
echo 'ok: release tags include the date and match the package version'
