#!/bin/sh
# Check the release date and package version before publication.
set -eu
cd "$(dirname "$0")/../.."
tag=${1:-}
if ! printf '%s\n' "$tag" | grep -Eq '^v[0-9]{4}\.(0[1-9]|1[0-2])\.(0[1-9]|[12][0-9]|3[01])\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'; then
  echo "$tag is not a release tag: use vYYYY.MM.DD.MAJOR.MINOR.PATCH with the package version" >&2
  exit 1
fi

cargo_version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
tag_version=$(printf '%s\n' "$tag" | cut -d. -f4-)
if [ "$tag_version" != "$cargo_version" ]; then
  echo "$tag names package version $tag_version, but Cargo.toml is at $cargo_version; use the version being released" >&2
  exit 1
fi
echo "ok: $tag names package version $cargo_version"
