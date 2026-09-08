# Changelog

## agent-notebook v2026.09.08.2

A Task belongs to who holds it. A team in which one person plans and the others do asked for reads that answer with the work each person holds, not with what each person wrote; this release is that ruling and the reads a planner and a developer need beside it. The packages move to 0.6.0: `--mine`, `--by <name>` and `scope: mine` no longer admit a Task by its author, and Status and a listing's JSON filter gain a field, so a script built on either reads the entries below first.

### New

- **Handing over at creation.** `add task --taken-by <name>` writes a Task and hands it over in one command, guarded like `edit --taken-by`; `add task --mine` takes it for the identity, and is refused with the fix named when the host knows nobody. A planner's session is one line per Task. (#76)
- **The pool is one read.** `--untaken` on `ready`, `list` and `graph` is the Tasks nobody holds, the ones anyone may take. It answers whose outright, so it outranks `scope: mine` and is refused beside `--by`, `--mine` or `--team`; the truncation hint carries it, and a listing's JSON filter carries `untaken`. (#76)
- **A narrowed Status counts the pool.** Under `scope: mine` a developer who held nothing saw a quiet notebook while the pool had work. The dashboard now prints `untaken: N — anb ready --untaken` when it is narrowed and the pool is not empty, a pool alone opens the gate, and `--json` carries `untaken.count`. The team's dashboard lists the pool in its queue and carries no count. (#76)

### Improved

- **Mine is what I hold.** `taken-by` names a Task's holder and `by` its author, and for a Task `mine` is `taken-by` and nothing else; `by` keeps meaning `mine` for a Question, a Note or a Decision. A Task you wrote and handed to a colleague is the colleague's in every read, and a Task nobody holds is nobody's, however many people wrote or planned it. `--by <name>` answers with a colleague's work the same way, and the dashboard's own-first order and marks read the holder. (#76)
- **The skill takes from the pool.** A session resumes the active Task, else takes the top of the user's queue, else the top of `ready --untaken`; the team paragraph is written around holder and author, and the worked session shows a hand-over at creation, the pool and a narrowed Status with an empty queue. (#76)

Packages in this release: `@supolka/agent-notebook@0.6.0` and its five platform packages at the same version.

## agent-notebook v2026.09.08.1

Status is where the work continues, and every listing narrows the same way. A team on one notebook asked for a read side that answers each of its questions once, and for a complexity revision before another flag was added; the revision is this release. The packages move to 0.5.0: `search` and `overview` are gone, `graph --ready` with them, Status no longer prints rules, epics or Debt rows, and the graph document is at version 3, so a script built on any of those reads the entries below first.

### New

- **One narrowing for every read.** `list`, `ready` and `graph` take `--for <hub>`, `--tag`, `--match <text>`, `--by <name>`, `--mine` and `--team`; `list` and `graph` also take `--type`, `--kind` and `--archive`. Each flag is a predicate over the same notebook, two flags ask for the intersection, and a narrowing changes what is shown, never what is true: a Task waiting on a colleague's stays blocked when their work is left out. A kind no type allows, a malformed tag or an empty text is refused with the vocabulary named, and a truncation hint carries every flag it was asked with. `anb <verb> --help` and the commands reference document the flags once. (#72)
- **The notebook says whose records a read answers with.** The config key `scope` is `team` by default; `scope: mine` narrows every read, Status and the session hook included, to the records the identity created or took, and `--team` widens one call. `--mine` and `scope: mine` without an identity are refused with the fix named. (#72)
- **The open Questions open the session.** Status carries a `questions[N]{id,age,by,title}` section, the reader's own first and then the oldest first, so a session sees the doubt it is about to work past. (#72)
- **`anb debt`.** Every sign of decay on its own line, in the clock table's order, bounded like every listing, with the same rows as `{count, debt}` in `--json`. (#72)

### Improved

- **Status is the work.** Active Tasks with the first one's last log line, review, held, the ready queue, the open Questions, and `debt: N — anb debt`. Rules and epics have no section and knowledge never opens the gate: a notebook of rules alone is quiet, and a rule is read before the work it binds with `list --type decision --kind rule`. Review and held lead with the reader's own and name another person's, and held carries a `taken-by` column. Under a narrowing a `by: <name> — anb status --team` line says so, and every hint on the dashboard carries the same `--by`. (#72)
- **`search` is `list --match`.** A text is one more predicate over id, title, tags, people and body, composable with every other flag; `--archive` reaches history, which `search` read by default. `overview` is `list --all`, and a type of it `list --type <t>`. (#72)
- **The graph is `list` with edges.** `graph` takes the listing's flags and no `--ready`: a ready Task waits on nothing live, so a graph of ready Tasks alone had no edges to draw, and every node still carries `ready` for a page to key on. The document's `slice` names each narrowing under its flag, and `v` is 3. (#72)
- **The dashboard opens nothing in the archive.** Epic progress is a read of its own: `ready --for <hub>` is an epic's queue, `list --for <hub> --archive` its whole membership, and a hub's graph node carries `closed/total` and `next`. (#72)

Packages in this release: `@supolka/agent-notebook@0.5.0` and its five platform packages at the same version.

## agent-notebook v2026.09.08

The notebook knows who is asking. Two issues from a team sharing one notebook close here: `by` was written and never read back, and a notebook of rules with no Task started every session blind to them. The packages move to 0.4.0: the ready table gains a column and a log entry is signed `by/via`, so a script that parsed either by position reads the header first.

### New

- **A Task is taken, never assigned.** `start` records who took the Task as `taken-by` and refuses a Task someone else took, a replay by another identity included, with the new `taken` refusal; `edit --taken-by` hands it over and `edit --clear taken-by` erases it. Nobody assigns work: `add` has no way to take a Task for someone else. (#67)
- **Your own records, and a colleague's.** `ready` and `list` take `--mine` and `--by <name>`, composable with `--for`, keeping the records an identity created or took; the ready table names who took each Task in a `taken-by` column, and `search` matches `by`, `via` and `taken-by`. (#67)
- **The identity the host acts under.** `ANB_BY` names it outright, for a checkout with no git identity or one whose git name is not the notebook's; otherwise git's `user.name` answers. `add` signs `by` with it, and `--mine` without one is refused with the fix named. (#67)

### Improved

- **Status opens on your work.** The caller's active Tasks lead the dashboard, and a Task another person took carries their name after its title; the hook gets the same. Every other section stays the whole project's. (#67)
- **A log entry names the person and the hand.** `comment` signs `by/via`, the form a cited record is attributed in, so the human stays in the trail beside the tool. (#67)
- **Rows carry their people.** `list`, `ready` and `search` rows, and Status `active` rows, carry `by` and `taken-by` in `--json` when the record has them. (#67)

### Fixed

- **A standing rule opens Status.** A notebook holding rule Decisions and no Task printed the quiet line, so the session hook delivered nothing of the law until the first Task existed; a live rule now counts as signal, and a shape Decision or a Note alone still leaves the notebook quiet. (#67)

Packages in this release: `@supolka/agent-notebook@0.4.0` and its five platform packages at the same version.

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
