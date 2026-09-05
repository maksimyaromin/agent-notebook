# Working in this repository

agent notebooks keeps a project's working memory as typed records in plain files: Tasks, Decisions, Notes and Questions under `.agent-notebook/`, one markdown file each, read and written by any coding agent through the CLI named `anb`. The Core is a Rust library behind a storage trait; the command line is a thin shell around it.

This file is for whoever changes that code, human or agent, and follows the [AGENTS.md](https://agents.md/) format so that any tool looking for one finds it. `CLAUDE.md` links it. Using the tool is the subject of [the book](./docs/index.md); the [README](./README.md) is its short form.

## Verifying a change

```sh
./scripts/check.sh              # what CI runs: format, clippy, tests, doctests, rustdoc, the renderings
pnpm install && pnpm docs:check # the docs job: the site builds, every page reachable, every link resolves
```

`rust-toolchain.toml` pins the Rust toolchain and `.nvmrc` the Node version; rustup and pnpm pick them up. Two committed trees are renderings of the binary and the gate fails when they drift: `anb skill .agents/skills/anb` regenerates the skill, `./scripts/docs-reference.sh` the two reference pages under `docs/reference/`.

## The notebook

This repository keeps its own backlog in `.agent-notebook/` and changes it only through the tool, never by editing the files:

```sh
cargo run --quiet -- status     # where the last session stopped
cargo run --quiet -- ready      # what can start now
cargo run --quiet -- --help     # every command
```

`cargo install --path crates/anb` puts `anb` on the path so the prefix can go. The working method is the shipped skill, [`.agents/skills/anb/SKILL.md`](./.agents/skills/anb/SKILL.md): open from Status, resume the active Task or start the top of the queue, log with `comment` as you go, file every doubt as a Question and every ruling as a Decision, close with a proof (`--note <report>` by default) and archive in the same breath, close Questions when they settle, hold with a reason, and leave `anb check` green. The skill keeps one Task in flight at a time; the tool does not enforce that, and Status shows the active line for a reason. A friction met with the tool itself is a finding: a comment on the Task it burdens, or a new Task or Question born `--from` it.

## Layout

| Path | What it holds |
|---|---|
| `crates/anb-core` | the record model, the line grammar, the notebook and its verbs, Status and Debt, check; no filesystem, git or network of its own |
| `crates/anb` | the command line: clap, the filesystem storage, the text and JSON renderers, recovery, the lock, setup, the skill generator |
| `.agents/skills/` | the two skills the tool installs into projects: `anb`, rendered from the binary, and `anb-atlas`, written by hand |
| `docs/`, `apps/docs` | the book and the Astro Starlight app that serves it; `tools/docs/check.mjs` is the docs check |
| `packages/` | the npm shim package and one package per platform; `scripts/release/` the release tooling |
| `.agent-notebook/` | this project's own notebook |

## Conventions

| Rule | Where it is owned |
|---|---|
| A test specifies one observable behaviour through the interface callers use; expected values come from the promise, never from running the code. | [Development](./docs/contributing/development.md) |
| A comment states a fact a stranger can check; a comment that restates the code, records a decision's history or defends it is deleted. | [Development](./docs/contributing/development.md) |
| Every text the tool prints, every skill it renders and every page of the book is plain engineering English: one word per concept across every surface, sentence-case headings, no dash as a connective in prose. The reply shapes are a contract of their own. | [Development](./docs/contributing/development.md), [Replies](./docs/reference/replies.md) |
| Committed text is self-contained: it never points at a file outside this repository, and it states a constraint rather than citing where it was decided. | this file |
| Code, comments, commit messages and docs are in English. In prose the project is agent notebooks; `anb` is the command, the crate names, the `ANB_` variables and the record ids. | this file |
| A commit message is a [conventional commit](https://www.conventionalcommits.org/en/v1.0.0/). A commit names its author and nobody else: no agent attribution, no `Co-Authored-By`, no `Generated with`, in any commit message or pull request text. | this file |
| main moves only by pull request with a green CI check, never by direct push. | this file |
| A markdown paragraph or list item is one physical line, however long; editors soft-wrap, hard wraps corrupt diffs. | this file |
| A new page under `docs/` joins `apps/docs/src/sidebar.js`; the docs check fails a page the sidebar misses and an entry that names no page. | [Development](./docs/contributing/development.md) |
| A behaviour change edits the page nearest that behaviour in the same pull request, and regenerates the skill and the reference pages when a command or a reply moved. | this file |

## Reading order

[What it is](./docs/start/what-it-is.md) states the three rules the tool holds. [Records and files](./docs/reference/records.md) is the model the Core enforces, and [Replies](./docs/reference/replies.md) the contract every command keeps. [Development](./docs/contributing/development.md) is the workspace and the gate. The notebook itself explains why things are the way they are: `cargo run --quiet -- overview` reads it as one page, and the archive holds every closed Task with its report.
