#!/bin/sh
# Publish the npm packages from a maintainer's machine, the way the first
# release of a package goes before Trusted Publishing exists:
#   scripts/release/publish.sh            a dry run of every package
#   scripts/release/publish.sh --publish  the real thing; npm asks for a one-time code when the account requires one
#
# It needs two lines in a git-ignored .env at the repository root and reads
# them from that file only, never from the environment:
#   NPM_TOKEN=…        a token allowed to publish the @supolka packages
#   NPM_PUBLISHER=…    the npm account the token belongs to
# The token is handed to npm through a temporary user config, so the user's
# own npm configuration plays no part, and the script refuses unless npm
# answers `whoami` with the expected account.
set -eu
cd "$(dirname "$0")/../.."
unset NPM_TOKEN NODE_AUTH_TOKEN NPM_CONFIG_USERCONFIG || true

mode=--dry-run
[ "${1:-}" = "--publish" ] && mode=

[ -f .env ] || { echo "no .env at the repository root; see the header of this script"; exit 1; }
token=$(sed -n 's/^NPM_TOKEN=//p' .env | tr -d '"' | tr -d "'" | head -1)
[ -n "$token" ] || { echo ".env has no NPM_TOKEN= line"; exit 1; }
expected=$(sed -n 's/^NPM_PUBLISHER=//p' .env | tr -d '"' | tr -d "'" | head -1)
[ -n "$expected" ] || { echo ".env has no NPM_PUBLISHER= line naming the account the token belongs to"; exit 1; }

userconfig=$(mktemp)
trap 'rm -f "$userconfig"' EXIT
chmod 600 "$userconfig"
printf '//registry.npmjs.org/:_authToken=%s\n' "$token" > "$userconfig"

who=$(npm whoami --userconfig "$userconfig" 2>/dev/null || true)
if [ "$who" != "$expected" ]; then
  echo "npm whoami answered '${who:-nobody}', not '$expected'; the token in .env does not belong to the expected account"
  exit 1
fi
echo "ok: publishing as $who"

sh scripts/release/check-versions.sh

for platform in darwin-arm64 darwin-x64 linux-x64 linux-arm64 win32-x64; do
  case "$platform" in win32-*) name=anb.exe ;; *) name=anb ;; esac
  binary="packages/agent-notebook-$platform/bin/$name"
  [ -f "$binary" ] || { echo "no binary at $binary; run scripts/release/fetch-binaries.sh <run-id> first"; exit 1; }
done

# Platform packages first, so the shim package never points at a version
# that does not exist yet.
for dir in packages/agent-notebook-darwin-arm64 packages/agent-notebook-darwin-x64 packages/agent-notebook-linux-x64 packages/agent-notebook-linux-arm64 packages/agent-notebook-win32-x64 packages/agent-notebook; do
  echo "== $dir"
  (cd "$dir" && npm publish --access public --userconfig "$userconfig" $mode)
done
[ -n "$mode" ] && echo "dry run complete; run with --publish to publish"
exit 0
