#!/bin/sh
# Refuse a release tag of the wrong shape:
#   scripts/release/check-tag.sh v2026.09.06.1
# A tag names the day of the release, vYYYY.MM.DD, and a later release on the same day appends .N, counting from 1:
# v2026.09.06 is the day's first release and v2026.09.06.1 its second. Exit 1 for anything else.
set -eu
tag=$1
if printf '%s\n' "$tag" | grep -Eq '^v[0-9]{4}\.(0[1-9]|1[0-2])\.(0[1-9]|[12][0-9]|3[01])(\.[1-9][0-9]*)?$'; then
  echo "ok: $tag names a day"
else
  echo "$tag is not a release tag: the day's first release is vYYYY.MM.DD and each further release that day appends .N, counting from 1" >&2
  exit 1
fi
