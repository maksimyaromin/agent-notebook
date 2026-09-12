#!/bin/sh
# Print the CHANGELOG.md entry of one release, the notes of its GitHub release:
#   scripts/release/changelog-notes.sh v2026.09.12.0.9.0
# The entry is the section whose heading ends with the tag. Exit 1 when there is none, so a tag without an entry is refused.
set -eu
cd "$(dirname "$0")/../.."
tag=$1
notes=$(awk -v tag="$tag" '
  /^## / { printing = ($NF == tag); next }
  printing { print }
' CHANGELOG.md)
if [ -z "$(printf '%s' "$notes" | tr -d '[:space:]')" ]; then
  echo "CHANGELOG.md has no entry for $tag" >&2
  exit 1
fi
printf '%s\n' "$notes"
