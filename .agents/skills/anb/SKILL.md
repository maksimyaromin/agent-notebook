---
name: anb
description: Use when working in a repository that has an .agent-notebook directory: when asked to continue a task or an epic, to pick the next piece of work, to record a decision, a doubt or a finding, to close work with its proof, or to say where the project stands. Also when a session starts and a status line beginning with active: was printed.
metadata:
  managed-by: anb
---

# anb

## Overview

`anb` keeps a project's working memory as typed records in `.agent-notebook/`: Tasks (open → active → review → closed), Decisions (active → superseded | retired), Notes (active → retired), Questions (open → closed). Every record has an id you type, such as `task.parser-fences` or `decision.no-mise-toml`, and a markdown file a human can read. Change the notebook only through `anb`, never by editing the files. The notebook is the developer's memory of the project, and keeping it clean is your job.

Every reply is short plain text an agent parses at a glance: `ok: <verb> <id> — <what changed>`, tables as `name[N]{fields}:` with comma rows, and refusals as `error[<code>]: <message>` followed by `try:` lines. A `try:` line is a command that runs as printed, so run one instead of guessing. Listings are bounded; `--all` lifts the bound, and `--json` gives the same data as compact JSON.

## When to use

- The session starts in a repository with `.agent-notebook/`, or a hook printed a line beginning with `active:`.
- Someone asks to continue, resume or pick up a task or an epic, or to take the next piece of work.
- You are about to record a decision, a doubt, a term, a fact or a finding. The notebook is where it goes, not a chat message or a stray file.
- Work is done and needs closing with its proof, or the developer asks where the project stands.

A repository without a notebook is outside this skill until `anb setup` wires one and `anb add` creates the first record. Editing record files is outside it always: every change is a verb.

## Quick reference

| Situation | Command |
|---|---|
| Where did the last session stop? | `anb status` |
| What can start now? | `anb ready`, or inside one epic `anb ready --for <hub>` |
| Take a Task into work | `anb start <id>` |
| Log progress | `anb comment <id> "<one line>"` |
| A doubt while working | `anb add question "<title>" --from <task>` |
| A ruling | `anb add decision "<title>" --kind rule` (or `shape`, `drift`) |
| A fact, a term, a guide | `anb add note "<title>" --kind fact` (or `term`, `guide`) |
| Hand work to a human | `anb submit <id>` |
| Close with proof, then file | `anb close <id> --note <report.md>`, then `anb archive <id>` |
| A Question settled | `anb close <question> --resolved-by <id>`, or `--reason "<why>"` |
| Pause, resume | `anb hold <id> --reason "<why>"`, `anb unhold <id>` |
| Find a record | `anb search <words>`, then `anb show <id>` |
| Verify the notebook | `anb check` |

Three references sit under `references/`, one per need, so open only the one you have. Before a verb you have not used, open [commands](references/commands.md): every verb with its flags and what each means. To see what a reply looks like before you parse one, open the [worked session](references/session.md): a notebook from empty to archived work, every reply as printed. When a refusal's `try:` line is not enough, open [refusals](references/refusals.md): every error code with its cause and repair, and the findings `anb check` raises.

## The session

1. Run `anb status`. The `active` line is where the last session stopped, with its last log entry. If a hook already printed it, do not print it again.
2. Resume the active Task. If none is active, take the top of `anb ready` and run `anb start <id>`.
3. Asked to continue an epic: find the hub with `anb search <words>`, which looks through ids, titles and tags. Then take the active Task inside it, or else the top of `anb ready --for <hub>`, and start it.

## The Task loop

- `anb start <id>` takes a Task into work, and takes it back from review.
- `anb comment <id> "<one line>"` as you go: where you stopped, what you decided, what you found. The next session resumes from the log rather than from scratch.
- Every friction met on the way becomes a record instead of a workaround: a doubt is `anb add question "<title>" --from <task>`, a ruling is `anb add decision`, a fact is `anb add note`.
- `anb submit <id>` hands the work to a human when one accepts it; the human's `anb start <id>` takes it back.
- `anb close <id> --note <report.md>` closes with the report ingested as a Note. That is the default proof because it travels with the notebook. The others are its equals: `--pr <url>`, `--sha <sha>`, `--report <path>` for a living document left where it lies, and `--no-proof` when the work is done and there is nothing to show. Work that will never happen ends with `anb close <id> --reason "<why>"`, which is legal from open.
- Run `anb archive <id>` right after the close. A closed record left live is a leftover somebody else has to find.

Hygiene is your job, not the developer's. Close Questions the moment they are settled: `anb close <question> --resolved-by <decision-or-task>` when the answer became a record, `--reason "<why>"` when it did not. Pause with `anb hold <id> --reason "<why>"` and resume with `unhold`; a hold without a reason is where work rots. Leave nothing for the developer to tidy.

## The shape of work

Work is almost never a flat sheet. An idea gets a hub Task tagged `epic`, and every Task born inside it is created `--from <hub>`. The hub is blocked by each child (`anb block <hub> <child>`), so it is not ready until the last child closes; then close and archive the hub like any Task. `anb ready --for <hub>` and `anb list --for <hub>` see one epic's work, and `anb status` shows every epic's progress and its next Task. Create children this way by default: origin is written at `add` time and cannot be added later.

## Knowledge

- Decisions carry a kind: `rule` for how we work, `shape` for a design ruling, `drift` for a deviation that stands until superseded. Replace one with `anb add decision "<title>" --supersedes <old>`; end one without a successor with `anb retire <id>`. The reply names a live Decision yours may conflict with. Read it before going on.
- Notes are curated knowledge corrected in place with `anb edit`; their kinds are `fact`, `term` and `guide`.
- In prose, a bare id is a reference and a backticked one is a quotation. Cite records by id in bodies and comments. A citation of an id that exists nowhere comes back as `dangling-mention`: a forward reference is legal, a typo is not.

## The user's own notebook

`--global` names the user's notebook in the home directory, for knowledge that outlives one repository. Decisions and Notes live there; Tasks and Questions are refused, because work stays in the project. A practice is a global Note tagged `skill`, addressed by name with `anb search <name> --global` and then `anb show <id> --global`, so "use my skill X" resolves through anb rather than through a pasted path. A project rule that stands against one of the user's cites the user's id in its body, and Status then names the pair.

## Before you stop

`anb check` must be green. It verifies every file and names the move that repairs each finding. Commit the notebook with the code it describes.

## Common mistakes

| Mistake | Instead |
|---|---|
| Editing a record file by hand | The verb that makes the change; `anb edit` for a field or the body |
| Closing without archiving | `anb archive <id>` right after `anb close` |
| A Question left open after its answer became a Decision | `anb close <question> --resolved-by <decision>` |
| A hold with no reason, or a Task parked in a chat message | `anb hold <id> --reason "<why>"` |
| A child Task added without its hub | `anb add task "<title>" --from <hub>`, then `anb block <hub> <child>` |
| Guessing after a refusal | Run the `try:` line as printed |
| Starting a second Task while one is active | Finish, submit or hold the active one first; Status shows one line for a reason |
