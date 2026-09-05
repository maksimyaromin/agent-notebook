---
title: Releasing
description: 'Build and publish the native binaries and npm launcher with matching versions.'
---

The npm package `@supolka/agent-notebook` supplies the `anb` launcher. It selects a native binary from an optional platform dependency. The packages cover macOS and Linux on x64 and arm64, and Windows on x64. npm downloads the packages during installation; no postinstall script fetches a binary.

## Keep versions together

The Cargo workspace, every npm package and the launcher's platform dependency pins use the same version. Check them before releasing:

```sh
sh scripts/release/check-versions.sh
```

The Release workflow runs this check before publishing.

## Publish a release

A release is named by its date, `v2026.09.05`, and the packages carry their own version. Bump the package version only when the packages change; a release that ships the same packages again is refused by the registry, and the workflow runs to that point without harm.

1. When the packages change, update `Cargo.toml` and every `packages/*/package.json`, including the launcher's `optionalDependencies`, and run the version check.
2. Write the release's entry in `CHANGELOG.md` under a heading that ends with the tag, `## anb v2026.09.05`: a paragraph on what the release means for a user, then `New`, `Improved` and `Fixed` with the pull requests in parentheses, and the package version the release ships. The workflow refuses a tag without an entry; `sh scripts/release/changelog-notes.sh v<date>` prints the entry it will use.
3. Merge through a pull request, then push `v<date>` on the merged commit.
4. Inspect the Release workflow. It builds the platform binaries, creates the GitHub release named after the tag with one archive per platform, a `SHA256SUMS` file and the changelog entry as its notes, checks that the manifests agree and smoke-tests the launcher with the Linux binary. It publishes the platform packages before the launcher.

The repository variable `RELEASE_DRY_RUN` controls publication. Unless its value is `false`, the workflow runs without publishing to npm.

## Bootstrap a package

The release process uses a local publish to create packages before configuring Trusted Publishing:

1. Push the tag and wait for the workflow's build artifacts. Record the run id.
2. Run `sh scripts/release/fetch-binaries.sh <run-id>` to download binaries into the platform packages.
3. Run `sh scripts/release/publish.sh` to inspect a dry run, then `sh scripts/release/publish.sh --publish` to publish.

The publish script documents its authentication inputs and checks the expected npm account. It publishes platform packages first so the launcher can resolve its dependencies.

After the initial publish, configure each package's Trusted Publisher for this repository and `release.yml`, then set `RELEASE_DRY_RUN` to `false`. The workflow uses OIDC for publication and enables provenance when the repository is public.
