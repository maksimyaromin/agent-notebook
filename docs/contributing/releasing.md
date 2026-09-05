---
title: Releasing
description: 'How a version reaches npm: one version everywhere, a tag, the Release workflow, and the first publish from a maintainer’s machine.'
---

The command ships on npm as `@supolka/agent-notebook`, a shim package whose `bin/anb.js` hands the command line to a binary that arrives as an optional dependency, one package per platform: `@supolka/agent-notebook-darwin-arm64`, `-darwin-x64`, `-linux-x64`, `-linux-arm64` and `-win32-x64`. Nothing is downloaded and nothing runs at install time, and `npx -y @supolka/agent-notebook` works from zero.

## One version everywhere

The Cargo workspace version is the version of every npm package and of the pins the shim puts on its platform packages. `scripts/release/check-versions.sh` fails when any of them disagree, and with a tag as its argument when the tag disagrees too. The Release workflow runs it before publishing.

## A release

1. Set the version in `Cargo.toml` and in every `packages/*/package.json`, including the shim's `optionalDependencies`; run `sh scripts/release/check-versions.sh` until it answers `ok`.
2. Merge through a pull request, then push the tag `v<version>` on the merged commit.
3. The Release workflow builds the binary on five runners, places each in its platform package, checks the versions against the tag, runs the shim against the Linux binary, and publishes the platform packages first and the shim last. While the repository variable `RELEASE_DRY_RUN` holds anything but `false`, every step runs and nothing reaches the registry.

## The first publish

Trusted Publishing needs the packages to exist, so the first publish is made from a maintainer's machine:

1. Push the tag and let the workflow build. Note its run id.
2. `sh scripts/release/fetch-binaries.sh <run-id>` downloads the five binaries into the platform packages.
3. `sh scripts/release/publish.sh` runs a dry run of every package; `sh scripts/release/publish.sh --publish` publishes, npm asking for the one-time code.

The publish token lives in the repository's git-ignored `.env` as `NPM_TOKEN=…`. The script reads it from that file and never from the environment, hands it to npm through a temporary user config so `~/.npmrc` plays no part, and refuses unless `npm whoami` answers with the account that owns `@supolka`.

After the first publish, add a Trusted Publisher on npm for this repository and the `release.yml` workflow, set `RELEASE_DRY_RUN` to `false`, and the workflow publishes by OIDC with no secret. Provenance statements are generated once the repository is public; the workflow derives the setting from the repository's visibility.
