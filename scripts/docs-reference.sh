#!/bin/sh
# The reference pages the binary writes: the command list and the refusal
# catalog come from `anb skill`, so the site cannot say what the tool does
# not do. `--check` fails when a committed page differs from the rendering.
set -eu
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cargo run --quiet --locked -- skill "$tmp" >/dev/null

# A skill reference becomes a docs page: Starlight frontmatter in place of the
# skill's, the title heading dropped (the page has its own), the contents
# list dropped (the site draws one).
page() {
  printf -- '---\ntitle: %s\ndescription: %s\n---\n\n' "$2" "$3"
  awk '
    /^---$/ && fence < 2 { fence++; next }
    fence < 2 { next }
    /^# / { next }
    /^## Contents$/ { skip = 1; next }
    skip && (/^- / || /^$/) { next }
    !begun && /^$/ { next }
    { skip = 0; begun = 1; print }
  ' "$1"
}
page "$tmp/references/commands.md" "Commands" "Every anb command with its arguments and flags, rendered from the binary." > "$tmp/commands.page"
page "$tmp/references/refusals.md" "Refusals and findings" "Every refusal code with its example and repair, and the check findings by severity, rendered from the binary." > "$tmp/refusals.page"

if [ "${1:-}" = "--check" ]; then
  status=0
  for name in commands refusals; do
    if ! diff -u "docs/reference/$name.md" "$tmp/$name.page" >/dev/null 2>&1; then
      echo "docs/reference/$name.md differs from the binary's rendering; run ./scripts/docs-reference.sh"
      status=1
    fi
  done
  exit $status
fi
cp "$tmp/commands.page" docs/reference/commands.md
cp "$tmp/refusals.page" docs/reference/refusals.md
echo "ok: docs/reference/commands.md and docs/reference/refusals.md rendered"
