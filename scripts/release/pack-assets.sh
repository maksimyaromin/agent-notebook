#!/bin/sh
# Pack the binaries a Release run built into one archive per platform, plus SHA256SUMS:
#   scripts/release/pack-assets.sh <tag> <binaries-dir> <out-dir>
# <binaries-dir> holds anb-<platform>/anb, or anb.exe on Windows, as actions/download-artifact lays them out.
set -eu
tag=$1
from=$2
mkdir -p "$3"
out=$(cd "$3" && pwd)
for platform in darwin-arm64 darwin-x64 linux-x64 linux-arm64 win32-x64; do
  case "$platform" in
    win32-*) (cd "$from/anb-$platform" && zip -q "$out/anb-$tag-$platform.zip" anb.exe) ;;
    *)
      # An artifact download drops the executable bit; the archive is where it is restored.
      chmod +x "$from/anb-$platform/anb"
      tar -czf "$out/anb-$tag-$platform.tar.gz" -C "$from/anb-$platform" anb
      ;;
  esac
done
(cd "$out" && if command -v sha256sum >/dev/null; then sha256sum anb-"$tag"-*; else shasum -a 256 anb-"$tag"-*; fi > SHA256SUMS)
