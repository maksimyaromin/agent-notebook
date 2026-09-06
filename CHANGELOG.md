# Changelog

## agent-notebook v2026.09.06.2

The third release of the day, and the first shaped by an adopter: seven issues were filed after the tool was set up on an existing project and its history replayed through the live verbs, and six of them close here. The packages move to 0.3.0: `anb setup` now refuses to wire an agent nobody named, so a script that ran it bare must name one.

### New

- **Setup wires only the agents named.** `anb setup --agent claude-code`, `--agent codex`, or `--agent agents-md` for any tool that follows the agents.md convention, repeatable; without the flag setup refuses and names the three. `--remove` takes the same flag, a file two agents read goes only when every reader is named, and a host directory setup emptied goes with its files. (#54)
- **A body from a file or a pipe.** `add` and `edit` take `--body-file <path>` beside `--body`, with `-` for standard input, and `close --note -` reads standard input by the same rule. (#52)
- **A link declared after the fact.** `edit --link` and `edit --unlink` splice links like tags, and a link naming a record must name one that exists. A Decision that cites another as context declares the relationship once, and `may-conflict` then names only the pair nobody has judged. (#53)
- **Debt as data.** Each row of `status --json` carries its signal's fields beside the printed line: the id and days of a clock, the pair of a `may-conflict`, the file and error count of an invalid record. (#50)

### Improved

- **A report's id is its Task's.** `close --note` mints `note.report-<task slug>`, guessable from the Task and never cut on a stopword; existing reports keep their ids. (#51)

### Fixed

- **The setup snippet is a paragraph of its own.** A guide ending with a list item no longer reads the snippet as part of that item, and removal leaves a guide ending on one newline as it was. (#49)

Packages in this release: `@supolka/agent-notebook@0.3.0` and its five platform packages at the same version.

## agent-notebook v2026.09.06.1

The second release of the day, and the first named by the same-day counter. Notes take three new kinds, the skills teach the author's own method from an idea to its proof, and the book and the README present the product in that voice. The packages move to 0.2.0: an older binary refuses the new Note kinds.

### New

- **Three kinds of Note for shaping work.** `idea` keeps a request with its source, its constraints and where the agreement stands; `model` holds a piece of the domain as the project understands it; `spec` states what a change is expected to do. They join `fact`, `term` and `guide`, and none of them turns knowledge into a Task. (#39)
- **The author method in the skill.** The `anb` skill starts a change from an idea Note, lets Tasks emerge `--from` it, keeps one Task active, cites what a record rests on and closes with evidence. The atlas skill applies the same method before it acts on review comments and tells a record's type from its kind. (#39)
- **The Ideas and Domain guides.** Two pages in the book on capturing a request and keeping a domain model, beside the guides on the session, Tasks and knowledge. (#39)
- **The project's mark.** The lockup at the top of the README, the mark beside the site title, the favicons and a brand directory with every variant. (#34, #38)

### Improved

- **The book in the author's voice.** The README and the landing page present the product, the contribution guide addresses developers who know their craft, and the release guide follows the workflow as it runs, with recovery when publication only partly succeeds. (#39)
- **A release tag is its day.** The first release of a day is `vYYYY.MM.DD`; each further release that day appends `.N`, counting from 1. The Release workflow refuses a tag of another shape before it creates a release or publishes a package. (#40)
- **The documentation build warns about nothing.** A declared translations collection, a 404 page of the site's own, and a chunk limit set just above the diagram grammar. (#36)

Packages in this release: `@supolka/agent-notebook@0.2.0` and its five platform packages at the same version.

## agent-notebook v2026.09.06

The first release published entirely by the repository: a tag builds every platform, creates the GitHub release and publishes the packages through npm's trusted publishing, with no token anywhere. Nothing changes for a user of `anb`; the packages move to 0.1.1 so the chain has something to publish.

### Improved

- **The skill module is two symmetric children.** `skill` owns what the installed skills share: the frontmatter mark, the list of installables, writing a skill's files and diffing them. `skill/anb` renders the `anb` skill from the binary and `skill/atlas` carries the hand-written atlas skill, one file per skill.

Packages in this release: `@supolka/agent-notebook@0.1.1` and its five platform packages at the same version.

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
