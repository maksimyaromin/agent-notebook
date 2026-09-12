# Working in this repository

agent-notebook keeps a project's working memory as typed records in plain files: Tasks, Decisions, Notes and Questions under `.agent-notebook/`, one Markdown file each. Anyone can read the files; agents make changes through the CLI named `anb`. The notebook connects current work to shared knowledge and external sources without replacing a team's ticket system or documentation. The Core is a Rust library behind a storage trait; the command line handles host integration and encoding.

This file is for whoever changes that code, human or agent, and follows the [AGENTS.md](https://agents.md/) format so that any tool looking for one finds it. `CLAUDE.md` links it. Using the tool is the subject of [the book](./docs/index.md); the [README](./README.md) is its short form.

## Verifying a change

```sh
./scripts/check.sh              # what CI runs: format, clippy, tests, doctests, rustdoc, the renderings
pnpm install && pnpm docs:check # the docs job: the site builds, every page reachable, every link resolves
```

`rust-toolchain.toml` pins the Rust toolchain and `.nvmrc` the Node version; rustup and pnpm pick them up. Two committed trees are renderings of the binary and the gate fails when they drift: `anb skill .agents/skills/anb` regenerates the skill, `./scripts/docs-reference.sh` the two reference pages under `docs/reference/`.

## The notebook

This repository keeps its working memory in `.agent-notebook/` and changes it only through the tool, never by editing the files:

```sh
cargo run --quiet -- recall     # session focus, current work and relevant knowledge
cargo run --quiet -- ready      # what can start now
cargo run --quiet -- --help     # every command
```

`cargo install --path crates/anb` puts `anb` on the path so the prefix can go. Follow the shipped skill, [`.agents/skills/anb/SKILL.md`](./.agents/skills/anb/SKILL.md). Open with Recall and resume the user's named Task or the session's remembered focus. Several Tasks can be active; the first row is not an instruction to switch work. Use separate session ids for parallel conversations and respect another session's claim.

Record the useful difference: a finding, a decision, a remaining uncertainty or a next step that another session would otherwise lose. Keep routine progress on the Task with `comment`; create a Note, Decision or Question when it deserves an independent lifetime. Close work with a short outcome and relevant evidence using `--body` or `--body-file`. Answer Questions when they settle, hold blocked work with a reason, and archive records when they leave the current scope. Archiving a Task does not archive its knowledge. Leave `anb check` green, and commit notebook changes only when the user authorizes commits.

Shared project meaning belongs in the project notebook. Personal practices belong in `--personal` for this project or `--global` across projects; neither audience silently supplies missing shared records. Team workflow extensions belong in `.agents/anb.md`, separate from the generated skill. A friction met with the tool itself is a finding: a comment on the Task it burdens, or a Task or Question born `--from` it when follow-up work is needed.

## Layout

| Path | What it holds |
|---|---|
| `crates/anb-core` | the record model and grammar, notebook operations, memory queries, Status, Debt and check; no filesystem, git or network of its own |
| `crates/anb` | the command line: clap, filesystem storage, one structured TOON/JSON projection, recovery, locking, sessions, hooks, setup and skill generation |
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
| Code, comments, commit messages and docs are in English. In prose the project is agent-notebook; `anb` is the command, the crate names, the `ANB_` variables and the record ids. | this file |
| A commit message is a [conventional commit](https://www.conventionalcommits.org/en/v1.0.0/). A commit names its author and nobody else: no agent attribution, no `Co-Authored-By`, no `Generated with`, in any commit message or pull request text. | this file |
| main moves only by pull request with a green CI check, never by direct push. | this file |
| A markdown paragraph or list item is one physical line, however long; editors soft-wrap, hard wraps corrupt diffs. | this file |
| A new page under `docs/` joins `apps/docs/src/sidebar.js`; the docs check fails a page the sidebar misses and an entry that names no page. | [Development](./docs/contributing/development.md) |
| A behaviour change edits the page nearest that behaviour in the same pull request, and regenerates the skill and the reference pages when a command or a reply moved. | this file |

## Reading order

[What it is](./docs/start/what-it-is.md) explains the product's boundaries. [Records and files](./docs/reference/records.md) is the model the Core enforces, and [Replies](./docs/reference/replies.md) the contract every command keeps. [Development](./docs/contributing/development.md) is the workspace and the gate. The notebook itself records the project's reasoning: `cargo run --quiet -- list --all --team` lists current records, `list --type decision --kind rule --team` selects standing rules, and `list --archive --all --team` includes archived work. Use `show <id> --all` to read a record's full context and outcome.
