#!/bin/sh
# Put a built binary where its platform package ships it:
#   scripts/release/place-binary.sh <platform> <path-to-binary>
# <platform> is one of darwin-arm64, darwin-x64, linux-x64, linux-arm64, win32-x64.
set -eu
cd "$(dirname "$0")/../.."
platform=$1
source=$2
case "$platform" in
  win32-*) name=anb.exe ;;
  *) name=anb ;;
esac
dir="packages/agent-notebook-$platform"
[ -d "$dir" ] || { echo "no platform package for $platform"; exit 1; }
[ -f "$source" ] || { echo "no binary at $source"; exit 1; }
mkdir -p "$dir/bin"
cp "$source" "$dir/bin/$name"
chmod 755 "$dir/bin/$name"
echo "ok: $dir/bin/$name"
