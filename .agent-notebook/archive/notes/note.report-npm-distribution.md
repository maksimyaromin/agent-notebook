---
id: note.report-npm-distribution
type: note
state: retired
title: Report: npm distribution
by: Maksim Yaromin
from: task.npm-distribution
created: 2026-09-05
updated: 2026-09-05
---

# npm distribution (2026-09-05)

Report for task.npm-distribution: `@supolka/agent-notebook` ships the `anb` binary through npm the way esbuild and turbo do and swc and biome did for Rust: a shim package and one package per platform, nothing downloaded and nothing run at install time.

## What shipped

```
packages/
  agent-notebook/                    @supolka/agent-notebook: bin/anb.js, the shim
  agent-notebook-darwin-arm64/       @supolka/agent-notebook-<platform>: package.json with os/cpu;
  agent-notebook-darwin-x64/         the binary lands in bin/ at release time and is git-ignored
  agent-notebook-linux-x64/
  agent-notebook-linux-arm64/
  agent-notebook-win32-x64/
scripts/release/
  check-versions.sh                  Cargo version == every package.json == the shim's pins (== the tag)
  place-binary.sh                    put a built binary where its platform package ships it
  fetch-binaries.sh                  download a Release run's five binaries into the packages
  publish.sh                         the first publish, from a maintainer's machine
.github/workflows/release.yml        tag v* → build on five runners → publish, platform packages first
docs/contributing/releasing.md       the procedure, on the site
```

The shim resolves `@supolka/agent-notebook-<platform>` from `process.platform` and `process.arch`, hands over the whole command line with inherited stdio, forwards the exit code and re-raises a signal; a missing platform package or an unsupported platform is a one-line message with the way out (`npm install --include=optional`, or `cargo install --git`). The shim package pins every platform package at its own version as an optional dependency, so a platform npm cannot install is skipped and the message says so.

## Publishing, and the token

The owner's rulings hold: the publish token lives only in the repository's git-ignored `.env`; `publish.sh` reads it from that file, never from the environment (it unsets `NPM_TOKEN` and `NODE_AUTH_TOKEN` first), hands it to npm through a temporary user config that is removed on exit so `~/.npmrc` and any global token play no part, and refuses unless `npm whoami` answers `maksimy`. It checks the versions, requires a binary in every platform package, and publishes the platform packages before the shim so the shim never points at a version that does not exist. Without `--publish` it is a dry run of every package.

The Release workflow builds on `macos-15`, `macos-15-intel`, `ubuntu-latest`, `ubuntu-24.04-arm` and `windows-latest`, uploads each binary as an artifact, and in a `publish` job on the `npm` environment places them, checks the versions against the tag, runs the shim against the Linux binary, and publishes with `--dry-run` while the repository variable `RELEASE_DRY_RUN` holds anything but `false`. The job has `id-token: write` and no secret: once the packages exist and a Trusted Publisher is registered, OIDC is all it needs. Provenance follows the repository's visibility, since npm generates none for a private one.

## The owner's steps

1. Push `v0.1.0` on main; the workflow builds and dry-runs. Note the run id.
2. `sh scripts/release/fetch-binaries.sh <run-id>`, then `sh scripts/release/publish.sh` for the dry run and `sh scripts/release/publish.sh --publish` for the real one, with the OTP prompt.
3. On npm, add a Trusted Publisher for `maksimyaromin/agent-notebook`, workflow `release.yml`, environment `npm`; set `RELEASE_DRY_RUN` to `false`.

## Verified

`npm publish --dry-run` through the script is green for all six packages, answering `ok: publishing as maksimy` with the token read from the file: the shim tarball carries `bin/anb.js`, `package.json` and `README.md`; each platform tarball its binary and manifest. The shim, resolved through a `node_modules` layout as npm lays it out, answers `anb 0.1.0` to `--version` and forwards a refusal's text and exit code. `check-versions.sh` passes with and without the tag. The docs check passes with the releasing page.

## Smoke check

Sonnet 5, once: no must-fix and no should-fix. It ran the shim through an installed layout (version, a refusal with its exit code, a success in a fresh home, the missing-package and unsupported-platform messages, stdin passthrough, a unicode argument with spaces round-tripped byte for byte, a signal re-raised as exit 143), packed every manifest, drove the three scripts through their refusals, ran the publish dry run with a bogus `NPM_TOKEN` in the environment and saw it publish as `maksimy` from the file with no `.npmrc` left anywhere, checked every pinned action commit against GitHub and the runner labels against the runner-images repository, ran actionlint clean, and confirmed the docs page and the site build. Two nits, one taken: the platform packages now carry `homepage` like the shim; the toolchain step installing clippy and rustfmt for a release build is the same line CI uses and stays. It also confirmed that `npm publish --dry-run` succeeds with no credentials at all, so the first tag push cannot fail the publish job while the dry-run gate holds.
