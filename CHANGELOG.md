# Changelog

## agent-notebook v2026.09.05

The first release. A project's working memory as typed records in plain markdown files, and one command, `anb`, that any coding agent runs to read and write them. Four kinds of record with lifecycles the tool enforces, a Status that fits a token budget, a check that names the repair for anything it finds, and a skill rendered from the binary so what agents learn cannot drift from what the tool does. Installs from npm with a native binary per platform, or with `cargo install`.

### New

- **Four kinds of record, one file each.** Tasks, Decisions, Notes and Questions live under `.agent-notebook/`: a fenced envelope of `key: value` lines the tool owns, then a body it never parses. A state moves only by command, no move deletes a file or frees an id, and the archive keeps every settled record together with its reports.
- **The verbs of a session.** `add`, `start`, `submit`, `close` with a proof or a reason, `reopen`, `hold` and `unhold` with the reason on record, `block` and `unblock` with a cycle refused the moment it is written, `comment` as the Task's log, `retire` for a Decision or Note without a successor. A Question closes into the record that settled it. (#7)
- **`ready` is the dispatch queue.** Open, unblocked, unheld Tasks, most urgent first, and `--for <hub>` narrows it to one epic. An epic is a hub Task: children are born `--from` it, and the hub waits on them until the last one closes.
- **Status within a token budget.** A session opens on the active Task with its last log line, the standing rules, what waits on hold and the debt the notebook owes. The composite degrades in a fixed order to stay under the budget, 1500 tokens by default, and a held Task waits in its own section instead of posing as the active one. (#9)
- **One reply shape for every command.** An `ok:` line with the verb, the id and what changed, or a refusal with its code, the fact and a command that runs as printed. Tables name their columns once, listings are bounded and `--all` lifts the bound, `--json` returns the same data everywhere, and a repeated command answers `(already)`. The vocabulary is one plain word per concept, judged term by term. (#7, #14, #23)
- **`check` names the repair.** It reads every file, names each line it cannot accept and the command that fixes it, so a hand edit or a bad merge never silently drops a record. `edit --clear` erases a field, `restore` brings an archived record back byte for byte, and `delete` removes a record born by mistake but refuses while anything cites it. (#4)
- **Two agents at once.** Writers and readers serialize on a lock at the notebook root, so nobody reads a cascade half finished.
- **`setup` wires the agents.** One line in `AGENTS.md` and `CLAUDE.md` between markers, a `SessionStart` hook for Claude Code and Codex that runs `anb status --hook`, and the skills where each agent looks for them. Running it again patches in place, and `--remove` takes out only what setup wrote. (#10)
- **A skill rendered from the binary.** `anb skill` prints the `anb` skill from the same definitions that print `--help`, with every example run on a scratch notebook, and CI diffs the committed copy against the rendering. Its references live under `references/`, and its common mistakes are the ones the tool makes easy. (#11, #12, #22)
- **The atlas skill.** `anb graph --json` prints the notebook as records and edges, and `anb-atlas` teaches an agent to draw it as one self-contained page: a legend that filters, a side panel for the record, and comments that come back as lines the agent turns into commands. (#13, #22)
- **Your own notebook.** `--global` names a notebook in the home directory for Decisions and Notes that outlive one repository; Tasks and Questions are refused there. The write paths and `check` read it too. (#8)
- **Installs from npm or cargo.** `@supolka/agent-notebook` carries the launcher and one package per platform, macOS and Linux on x64 and arm64 and Windows on x64, with no postinstall download; `cargo install` builds from the repository. A tag builds every platform and publishes a GitHub release with the archives and their checksums. (#17, #18, #27)
- **The book.** Documentation at agent-notebook.supolka.dev, with the command and refusal reference pages rendered from the binary and checked in CI, and a README with one default adoption path. (#15, #16, #20, #21, #26)

Packages in this release: `@supolka/agent-notebook@0.1.0` and its five platform packages at the same version.
