#!/bin/sh
# Bring the binaries a Release workflow run built into the platform packages,
# for a publish from this machine:
#   scripts/release/fetch-binaries.sh <run-id>
# The run id is in the Actions tab, or `gh run list --workflow release.yml`.
set -eu
cd "$(dirname "$0")/../.."
run=$1
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
gh run download "$run" --dir "$tmp"
for platform in darwin-arm64 darwin-x64 linux-x64 linux-arm64 win32-x64; do
  case "$platform" in win32-*) name=anb.exe ;; *) name=anb ;; esac
  sh scripts/release/place-binary.sh "$platform" "$tmp/anb-$platform/$name"
done
