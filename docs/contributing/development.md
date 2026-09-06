---
title: Development
description: 'The design boundaries, evidence and working method behind changes to agent-notebook.'
---

I use agent-notebook to manage work on this project. The backlog and design decisions are in [the repository's notebook](https://github.com/maksimyaromin/agent-notebook/tree/main/.agent-notebook), along with closed Tasks and their reports. If you want to understand why something works the way it does, those records are a useful place to start.

## Contributing

I am interested in where the tool makes your work harder than it needs to be. A command may leave out something the agent needs, or the default skill may keep doing something your team does differently. A command and its reply, or an example from a session, will help explain the problem.

Contributions to the skills matter as much as changes to the CLI. If you have rewritten part of the workflow and used it, explain what you changed and how it worked. It may be useful as an alternative even if it does not become the default. The [customization guide](../guides/customization.md) describes the changes the tool already supports.

For work in this repository, start with:

```sh
cargo run --quiet -- status
cargo run --quiet -- search proof
```

Search includes the archive. Read the relevant Decisions and reports before reopening a design question: they record the constraint and the alternatives considered. When a constraint changes, record the new decision and its reason. The [session guide](../guides/session.md) covers the work loop; `AGENTS.md` contains the repository conventions.

## Where behavior belongs

Record validity belongs in the Core; working conventions belong in skills. For example, a dependency cycle makes the notebook inconsistent, but several active Tasks may be what a team needs. Before adding a restriction to the CLI, consider whether it is necessary for the records to remain valid or whether it expresses a preference about how to work.

`crates/anb-core` implements the grammar, record model, operations, queries and validation. Its `Storage` trait exchanges strings and relative paths. The in-memory adapter lets tests exercise the same notebook operations as a filesystem host, without a temporary repository or a shell. The Core receives dates and identity from its caller.

`crates/anb` supplies filesystem storage, command parsing, rendering, locks, git queries and agent setup. Concurrency belongs here: the host protects a whole operation, including changes to several files. Per-file atomic writes alone would let a concurrent reader mistake an unfinished archive move for a duplicate record.

This separation leaves room for other hosts to reuse the Core. A proposed host still has to define its storage and concurrency behavior; the trait does not provide transactions or synchronization.

## What a change must preserve

Changes have to account for existing notebook files and for agents using the command replies. These are the constraints I pay particular attention to:

- Preserve bytes outside the intended edit. The grammar retains raw spans, including malformed input, so a correction does not silently rewrite the rest of a file.
- Keep repeats and recovery explicit. Repeating an applied state transition leaves the record unchanged. A storage failure during a multi-file operation needs a recoverable outcome, not an assumed rollback.
- Keep enough information in a bounded reply to continue. Counts distinguish omitted rows from empty results; refusal codes and repair suggestions tell the caller what happened and what it can do next.
- Keep ordinary session reads focused on live work. Archive size and dependency shape both matter when measuring a query; a small live notebook with deep archived lineage can expose costs that a flat fixture misses.

Rust gives the record states and outcomes exhaustive matches and typed errors. The native binary also keeps the installed CLI independent of a language runtime; npm is a distribution option. These properties suit a tool invoked repeatedly during a session, but correctness still depends on the behavior we test.

## Tests and checks

Core tests exercise notebook operations through `Storage`. CLI tests run the binary against temporary directories. The negative corpus checks malformed input, and property tests cover preservation and lifecycle behavior. A regression test should name the observable failure and fail when the defect is present. Expected results come from the contract; unchanged behavior should survive a refactor without rewriting its tests.

Use the toolchains pinned in `rust-toolchain.toml` and `.nvmrc`. The code gate is:

```sh
./scripts/check.sh
```

It runs formatting, clippy with warnings as errors, tests, doctests and rustdoc, then checks generated files for drift. For documentation:

```sh
pnpm install --frozen-lockfile
pnpm docs:check
```

The docs check covers sidebar membership, source links, the site build and local links in the rendered pages. `pnpm docs:dev` serves the site locally.

## Documentation and skills

A command change affects its help, replies, installed references and the guide where someone learns to use it. `crates/anb/src/skill/anb.rs` renders the workflow skill and executes its worked session against a scratch notebook. The main skill contains the working method, including planning and domain modeling; separate references contain command syntax, worked replies and refusals. The atlas skill is authored under `.agents/skills/anb-atlas/` and bundled by `skill/atlas.rs`.

Regenerate the CLI-owned texts after changing their source:

```sh
cargo run --quiet -- skill .agents/skills/anb
./scripts/docs-reference.sh
```

`docs/reference/commands.md` and `docs/reference/refusals.md` are generated. Other pages are authored in `docs/` and served by the Starlight app in `apps/docs`. A new page joins `apps/docs/src/sidebar.js`.

Generated references catch syntax and example drift. They do not prove that a workflow instruction helps an agent. Read that instruction against the behavior it teaches, and use an actual session failure to decide what needs adding or removing. Cases from this project are in the notebook. Examples from other teams help check whether an instruction also works outside my own routine.
