---
name: anb
description: Work a project's notebook — its Tasks, Decisions, Notes and Questions — through the anb CLI: resume the active Task, take the next ready one, record a decision or a question, close work with its proof, and leave the notebook clean. Use whenever the session touches .agent-notebook, when asked to continue a task or an epic, and before recording a decision, a doubt or a finding.
metadata:
  generated: anb
---

# The notebook

`anb` keeps a project's working memory as typed records in `.agent-notebook/`: **Tasks** (open → active → review → closed), **Decisions** (active → superseded | retired), **Notes** (active → retired), **Questions** (open → closed). Every record has an id you type — `task.parser-fences`, `decision.no-mise-toml` — and a markdown file a human can read. Mutate the notebook only through `anb`; never edit the files.

Every reply is short plain text an agent parses at a glance: `ok: <verb> <id> — <what changed>`, tables as `name[N]{fields}:` with comma rows, and refusals as `error[<code>]: <message>` followed by `try:` lines. **A `try:` line is a command that runs as printed** — run one instead of guessing. Listings are bounded; `--all` lifts the bound, `--json` gives the same data as compact JSON. Three references sit beside this file, one per need, so open only the one you have: before a verb you have not used, [commands](commands.md) — every verb with its flags and what each means; to see what a reply looks like before you parse one, the [worked session](session.md) — a notebook from empty to archived work, every reply as printed; when a refusal's `try:` line is not enough, [refusals](refusals.md) — every error code with its cause and repair, and the findings `anb check` raises.

## The session

1. `anb status`. The **active** line is where the last session stopped, with its last log entry. If a hook already printed it, do not print it again.
2. Resume the active Task. If none is active, take the top of `anb ready` and `anb start <id>`.
3. Asked to *continue an epic*: find the hub — `anb search <words>` over ids, titles and tags — then take the active Task inside it, else the top of `anb ready --for <hub>`, and start it.

## The Task loop

- `anb start <id>` takes a Task into work (also back from review).
- `anb comment <id> "<one line>"` as you go: where you stopped, what you decided, what you found. The next session resumes from the log, not from scratch.
- Every friction met on the way is a record, never a workaround: a doubt is `anb add question "<title>" --from <task>`, a ruling is `anb add decision`, a fact is `anb add note`.
- `anb submit <id>` hands the work to a human when one accepts it; the human's `anb start <id>` takes it back.
- `anb close <id> --note <report.md>` closes with the report ingested as a Note — the default proof, because it travels with the notebook. Equals: `--pr <url>`, `--sha <sha>`, `--report <path>` (a living document left where it lies), `--no-proof` (done, nothing to show). Work that will never happen ends with `anb close <id> --reason "<why>"`, legal from open.
- Run `anb archive <id>` right after the close. A closed record left live is a leftover somebody else has to find.

**Hygiene is your job, not the developer's.** Close Questions the moment they are settled — `anb close <question> --resolved-by <decision-or-task>` when the answer became a record, `--reason "<why>"` when it did not. Pause with `anb hold <id> --reason "<why>"` and resume with `unhold`; a hold without a reason is where work rots. Leave nothing for the developer to tidy.

## The shape of work

Work is almost never a flat sheet. An idea gets a **hub** Task tagged `epic`; every Task born inside it is created `--from <hub>`, and the hub is blocked by each child — `anb block <hub> <child>` — so it is not ready until the last child closes; then close and archive the hub like any Task. `anb ready --for <hub>` and `anb list --for <hub>` see one epic's work; `anb status` shows every epic's progress and its next Task. Create children this way by default, not as an option: origin is written at `add` time.

## Knowledge

- **Decisions** carry a kind: `rule` (how we work), `shape` (a design ruling), `drift` (a deviation that stands until superseded). Replace one with `anb add decision "<title>" --supersedes <old>`; end one without a successor with `anb retire <id>`. The reply names a live Decision yours may conflict with — read it before going on.
- **Notes** are curated knowledge corrected in place (`anb edit`), kinds `fact`, `term`, `guide`.
- In prose, a bare id is a reference and a backticked one is a quotation. Cite records by id in bodies and comments; a citation of an id that exists nowhere is nudged back as `dangling-mention` — a forward reference is legal, a typo is not.

## The user's own notebook

`--global` names the user's notebook in the home directory: knowledge that outlives one repository. Decisions and Notes live there; Tasks and Questions are refused, work stays in the project. A practice is a global Note tagged `skill`, addressed by name — `anb search <name> --global`, then `anb show <id> --global` — so "use my skill X" resolves through anb, never through a pasted path. A project rule that stands against one of the user's cites the user's id in its body; Status then names the pair.

## Before you stop

`anb check` must be green — it verifies every file and names the move that repairs each finding. Commit the notebook with the code it describes.
