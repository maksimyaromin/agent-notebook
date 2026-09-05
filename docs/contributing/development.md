---
title: Development
description: 'Workspace structure, verification, and the conventions for code and documentation changes.'
---

## The workspace

`anb-core` owns the record model, grammar, notebook operations, queries and validation. It receives storage through a trait and has no filesystem, git or network access. Tests use an in-memory adapter; the CLI supplies filesystem storage.

`anb` owns command-line parsing, text and JSON rendering, filesystem operations, locking, git reconciliation, setup and skill generation. Keep behavior in the layer that owns it: a record invariant belongs in the Core, while a platform or shell concern belongs in the CLI.

The Rust toolchain is pinned in `rust-toolchain.toml`. Use the Node version in `.nvmrc` for documentation work.

## Verify a change

```sh
./scripts/check.sh
```

This is the CI gate. It checks formatting, clippy with warnings as errors, tests, doctests and rustdoc. It also compares the committed skill and generated reference pages with the binary's output.

For documentation changes, run:

```sh
pnpm install
pnpm docs:check
```

The docs check verifies sidebar coverage and relative links, builds the site, then checks that local links in the built pages resolve. `pnpm docs:dev` serves the site locally.

## Tests

A test specifies one observable behavior in a named situation through the interface callers use. Expected values come from the promised behavior, never from running the implementation. A refactor that preserves behavior should preserve the test.

Core tests exercise notebook operations through the storage interface. CLI tests run the binary against temporary directories. Property tests verify grammar and record behavior, including byte preservation when unchanged files are parsed and rendered.

## Text

Write code, comments and documentation in English. Use one word per concept across the CLI, skills and book. Headings use sentence case; prose uses no dash as a connective. Exact command output follows the [reply contract](../reference/replies.md).

Comments explain facts a reader cannot derive from the code, such as an external constraint or a failure mode. Remove comments that restate the implementation, narrate a change, or defend a design choice. Record decisions and their history in the notebook.

A Markdown paragraph or list item occupies one physical line. Use editor soft wrapping. Committed text must be self-contained: describe the constraint without pointing to material outside this repository.

## The notebook

Read `cargo run --quiet -- status` before working. The project's Tasks, Decisions and Questions live in `.agent-notebook/`; closed work and its reports are in the archive. Change these records through the CLI. [The session guide](../guides/session.md) describes the working method, and the repository's `AGENTS.md` gives contributor instructions.

## Documentation sources

The Astro Starlight app in `apps/docs` reads the Markdown files in `docs/`. The same files are readable on GitHub. Add new pages to `apps/docs/src/sidebar.js`; the check rejects both missing pages and pages unreachable from the sidebar.

A behavior change updates the nearest guide or reference in the same pull request. When commands or replies change, regenerate the installed skill and reference pages:

```sh
cargo run --quiet -- skill .agents/skills/anb
./scripts/docs-reference.sh
```

`docs/reference/commands.md` and `docs/reference/refusals.md` are generated. Edit their source in the binary, then regenerate them. The other book pages are authored directly.

## Submit a change

Use a conventional commit message in English. Commits name their author only, without agent attribution or co-author trailers. Main changes through pull requests with a green CI check; do not push changes directly to it.
