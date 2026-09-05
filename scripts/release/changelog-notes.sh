#!/bin/sh
# Print the CHANGELOG.md entry of one version, the notes of its GitHub release:
#   scripts/release/changelog-notes.sh v0.1.0
# Exit 1 when the version has no entry, so a tag without one is refused.
set -eu
cd "$(dirname "$0")/../.."
version=${1#v}
notes=$(awk -v version="$version" '
  /^## \[/ { printing = index($0, "## [" version "]") == 1; next }
  /^\[[^]]*\]: / { printing = 0 }
  printing { print }
' CHANGELOG.md)
if [ -z "$(printf '%s' "$notes" | tr -d '[:space:]')" ]; then
  echo "CHANGELOG.md has no entry for $version" >&2
  exit 1
fi
printf '%s\n' "$notes"
