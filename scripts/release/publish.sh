#!/bin/sh
# Publish the npm packages from this machine, the way the first release goes
# before Trusted Publishing exists:
#   scripts/release/publish.sh            a dry run of every package
#   scripts/release/publish.sh --publish  the real thing, npm asking for the OTP
#
# The token comes from this repository's git-ignored .env, line NPM_TOKEN=…,
# read from the file and never from the environment: the shell may carry a
# token of the same name for another account and another scope, and that one
# must never publish here. The token is handed to npm through a temporary
# user config, so the user's own ~/.npmrc plays no part, and the script
# refuses unless npm answers `whoami` with the account that owns @supolka.
set -eu
cd "$(dirname "$0")/../.."
unset NPM_TOKEN NODE_AUTH_TOKEN NPM_CONFIG_USERCONFIG || true

owner=maksimy
mode=--dry-run
[ "${1:-}" = "--publish" ] && mode=

[ -f .env ] || { echo "no .env in the repository root; put the publish token there as NPM_TOKEN=…"; exit 1; }
token=$(sed -n 's/^NPM_TOKEN=//p' .env | tr -d '"' | tr -d "'" | head -1)
[ -n "$token" ] || { echo ".env has no NPM_TOKEN= line"; exit 1; }

userconfig=$(mktemp)
trap 'rm -f "$userconfig"' EXIT
chmod 600 "$userconfig"
printf '//registry.npmjs.org/:_authToken=%s\n' "$token" > "$userconfig"

who=$(npm whoami --userconfig "$userconfig" 2>/dev/null || true)
if [ "$who" != "$owner" ]; then
  echo "npm whoami answered '${who:-nobody}', not '$owner'; the token in .env is not the @supolka publish token"
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
