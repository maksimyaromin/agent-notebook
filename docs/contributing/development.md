---
title: Development
description: 'The workspace, the gate, the tests, and how the project dogfoods its own notebook.'
---

## The workspace

Two crates. `anb-core` is the record model, the line grammar, the notebook and its verbs, Status and Debt, and `check`, behind a storage trait with no filesystem, git or network of its own; a local MCP server or another host reuses it unchanged. `anb` is the command line around it: clap, the filesystem storage, the text and JSON renderers, the recovery payloads, the lock, setup and the skill generator.

```sh
./scripts/check.sh
```

The gate runs `cargo fmt --check`, `cargo clippy` with warnings as errors, every test, the doctests, the rustdoc build with warnings as errors, the skill drift check (`anb skill .agents/skills/anb --check`) and the docs reference check (`./scripts/docs-reference.sh --check`). CI runs the same script on every pull request and every push to main.

## Tests

A test specifies one observable behaviour in one named situation through the interface callers use, so a behaviour-preserving change leaves the suite green. Expected values come from the promise, never from running the code. The Core is tested through the notebook API on an in-memory storage; the CLI end to end through the binary on a temporary directory; the grammar and the record model also by property tests over generated notebooks, which hold the byte-exact round-trip.

## Text

Every text the tool prints, and every skill it renders, is read by a person as well as an agent. The reply shapes are the contract in [Replies](../reference/replies.md); the words are plain engineering English, one word per concept across every surface, sentence-case headings, and no dash as a connective in prose. The same rule holds for the book and for every comment in the code.

A comment states a fact a stranger can check and the code cannot say for itself, usually a why. A comment that restates the code, records the history of a decision or defends it against a reviewer is deleted on sight; the notebook is where decisions and their history live.

## The notebook

This repository keeps its own backlog in `.agent-notebook/` at the root and changes it only through the tool. Every task, decision and question of the project is there, closed tasks in the archive with their reports as Notes. Reading it is the fastest way to learn why something is the way it is.

## The docs

The book lives in `docs/` and is rendered by the Astro Starlight app in `apps/docs`, which reads the tree where it lies, so GitHub renders the same files. Two pages are written by the binary: `./scripts/docs-reference.sh` renders the commands and the refusals from `anb skill`, and the gate fails when a committed page differs. `pnpm docs:check` verifies that the sidebar reaches every page and names no page that does not exist, that every relative link resolves, and that the built site references only files the build wrote; `pnpm docs:dev` serves it locally.
