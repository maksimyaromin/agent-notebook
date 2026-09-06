---
title: Releasing
description: 'Release artifacts, package versions and recovery when publication only partly succeeds.'
---

The [Release workflow](https://github.com/maksimyaromin/agent-notebook/blob/main/.github/workflows/release.yml) builds native binaries, attaches archives and checksums to a GitHub release, and publishes npm packages through Trusted Publishing. GitHub releases and npm publication are separate jobs after the build. Success on one does not establish success on the other.

## What ships

The npm package `@supolka/agent-notebook` supplies a launcher that selects a native binary from an optional platform dependency. The packages cover macOS and Linux on x64 and arm64, and Windows on x64. npm downloads the packages during installation; no postinstall script fetches a binary.

[GitHub releases](https://github.com/maksimyaromin/agent-notebook/releases) provide the binaries directly, one archive per platform, with `SHA256SUMS`. These do not require Node.js. The package and source versions agree, while a release tag names a date: for example, `v2026.09.06` ships package version `0.1.1`.

## Prepare the release

Update the Cargo workspace version, every npm package version and the launcher's `optionalDependencies` pins together. Then check their agreement:

```sh
sh scripts/release/check-versions.sh
```

Write the release entry in `CHANGELOG.md` under a heading such as `## agent-notebook v2026.09.06`. Start with what the release changes for a user, use the applicable `New`, `Improved` and `Fixed` sections with pull request references, and name the package version. The workflow extracts this entry as the release notes and refuses a tag without one. Inspect the extraction before tagging:

```sh
sh scripts/release/changelog-notes.sh v2026.09.06
```

Tag the reviewed commit with the date tag and push that tag. The workflow builds each platform, packages the release assets, checks version agreement and runs the npm launcher against the Linux binary. It publishes platform packages before the launcher so a newly installed launcher can resolve its dependencies.

The repository variable `RELEASE_DRY_RUN` must equal `false` for npm publication. A manual workflow run also has a dry-run input, enabled by default. That input controls npm publication; the GitHub release job runs for tag refs independently of it. Published npm versions cannot be reused, so a tag that should publish packages needs an unpublished package version.

## Check the outcome

Check the build, GitHub release and npm publish jobs separately. Confirm that the release has all platform archives and checksums and that the launcher and platform packages are available at the intended version. From an empty directory, run the published launcher at that explicit version:

```sh
npx -y @supolka/agent-notebook@0.1.1 --version
```

The numbers above illustrate a published release; substitute the version being released.

A rerun keeps an existing GitHub release and replaces its assets. npm publication has no equivalent resume behavior: the loop starts from the first platform package and stops on failure, including an already-published version. If publication stopped halfway, inspect which packages reached the registry before choosing a recovery. The workflow cannot roll them back, and rerunning the whole job is not a way to skip them.

## Bootstrap a new package

The existing packages use Trusted Publishing. Adding a package requires an initial publish before configuring its Trusted Publisher:

1. Build through the workflow and record the run id.
2. Use `sh scripts/release/fetch-binaries.sh <run-id>` to place those binaries in their package directories.
3. Read `scripts/release/publish.sh` for its authentication inputs and account check. Run it without flags to inspect the dry run; `--publish` performs publication.
4. Configure the new package's Trusted Publisher for this repository and `release.yml`, with the `npm` environment used by the workflow.

The local publish script walks all package directories too. Before using it when some versions already exist, account for the same partial-publication constraint. Normal releases use the workflow's OIDC identity; they do not require a stored npm publish token.

## Publish the book

Documentation deploys independently of binary releases. `pages.yml` builds changes to the book and site on main and uploads `apps/docs/dist` to Cloudflare Pages at [agent-notebook.supolka.dev](https://agent-notebook.supolka.dev). Pull requests run the documentation checks without deploying. The workflow supports a manual redeploy and reports when missing deployment credentials cause it to build without uploading.
