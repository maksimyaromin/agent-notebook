---
name: anb
description: Use when working in a repository with .agent-notebook, capturing or shaping an idea, modeling a domain, planning or continuing Tasks and epics, recording project knowledge, reviewing status, or when a session hook reports active work.
metadata:
  managed-by: anb
---

# anb

## Overview

Keep the problem, the reasoning and the work connected so another session can continue without reconstructing the conversation. Use the CLI for every notebook change; it maintains record state and relationships together.

## When to use

Use this method when capturing a request, shaping an idea, maintaining domain knowledge, planning delivery or continuing project work. For a status question, read and answer; a read does not need a new Task. Follow the user's chosen notebook location, workflow and sharing policy.

## Core pattern

Start a new change with an `idea` Note, or resume the existing idea. Keep the source, intended improvement, constraints, agreement status and next uncertainty in its body. Link the source with `--link "doc <path-or-url>"`; record missing evidence explicitly. Capturing a request does not authorize implementation or changes to its source.

Keep the idea when Tasks emerge: one proposal can lead to several deliveries. Create records `--from` what produced them, cite supporting records by bare id, and use `block` for execution prerequisites. An origin answers why a record exists; a mention supplies context; a dependency controls readiness; a `--link "<kind> <id>"` declares a relation of your own, such as the schema a document follows, and `show`, `graph` and `--for` walk it back from the record it names.

For an agreed small change, the idea and one Task are enough:

```sh
anb add note "Name the CSV download" --id note.csv-download --kind idea --via codex --body "Agreed: rename Export to Download CSV so the label states the format. Preserve behavior and file contents. Implementation is queued for later."
anb add task "Rename the CSV download button" --id task.csv-download --from note.csv-download --via codex --body "Implement note.csv-download. Verify the label and that the same action produces unchanged CSV content."
anb check
```

These explicit ids make the example runnable. In ordinary work, use the ids returned by the CLI, including collision suffixes. Add `--via` to every agent `add` and `comment`, using your actual tool name, such as `codex` or `claude-code`. Leave `by` to the accountable person: the CLI signs it from `ANB_BY` or the git identity, and a comment is signed `by/via`, so the person stays in the trail beside the tool. Other verbs do not accept `--via`.

## Quick reference

Choose by what a later reader needs, with an explicit `--kind` for Notes and Decisions.

| Need | Record | Why keep it separately |
|---|---|---|
| Deliver or investigate a checkable result | Task | Progress and completion belong to the work |
| Settle an uncertainty that changes the work | Question | An unanswered choice must remain visible |
| Preserve an agreed requirement | Decision `rule` | Later work must respect its scope and reason |
| Explain a chosen design | Decision `shape` | Alternatives and the deciding constraint prevent repeated debate |
| Allow an agreed exception | Decision `drift` | The affected rule and revisit condition bound the departure |
| Reuse an observation | Note `fact` | Evidence and limits distinguish a finding from a guess |
| Define a word in context | Note `term` | Ambiguous vocabulary changes how requirements are read |
| Repeat a procedure | Note `guide` | Conditions and verification make it reusable |
| Develop a possibility | Note `idea` | Motivation outlives any one delivery |
| Explain a domain | Note `model` | Ownership, relationships and invariants need more than definitions |
| Specify expected behavior | Note `spec` | Scope, exclusions and acceptance criteria guide delivery |

## Working method

### Orient

Read `anb status` unless the hook supplied it. Status is the work: the active Tasks with the last log line of the first, work in review or on hold, the queue, the open Questions and a count of Debt. Follow the requested subject; otherwise resume the active Task, or take the top of `ready`; when the user's own queue is empty and Status counts the pool on its `untaken:` line, take the top of `ready --untaken`. Before the work, read the standing rules with `anb list --type decision --kind rule` and open the ones its subject touches, then the knowledge the Task cites and the relevant code. Search before creating records: `anb list --match <text>` matches ids, titles, tags, the people named in the envelope and bodies, and `--archive` reaches history. Use `show --all` for a truncated body and narrowed lists for larger work; loading the whole notebook obscures the immediate decision.

Every listing narrows the same way: `--for <hub>`, `--tag`, `--match <text>`, `--by <name>`, `--mine`, `--team` and `--untaken` compose on `list`, `ready` and `graph`, and `--type`, `--kind` and `--archive` on `list` and `graph`, each answering with the records every flag admits. `list --type note --tag domain-model` is the domain language; `ready --tag parser` is one area's queue; `anb debt` is the Debt that Status counts.

Several people can share one notebook, and a Task belongs to whoever holds it: `taken-by` names the holder, `by` the author, and a Task nobody holds is nobody's, whoever wrote it. `start` takes an untaken Task for the user and refuses one another person holds. A planner hands a Task over as it is written with `add task --taken-by <name>`, or later with `edit <id> --taken-by <name>`, both decided by the user, never by the agent; `add task --mine` takes a Task for the user, for a follow-up the user will do. Status lists the user's own active Tasks first and marks another person's with their name, so resume only an unmarked line. In `ready`, the `taken-by` column names each Task's holder, and `ready --untaken` is the pool, the Tasks anyone may take. A Question put to a person with `add question --to <name>` and a review handed to one with `submit --to <name>` wait on that person, and their Status shows them beside their own; the `to` column of the review and questions tables says whom each waits on. A notebook whose config sets `scope: mine` answers every read with the Tasks the user holds, the records the user wrote and the records waiting on the user; Status then counts the pool on its `untaken:` line, and `--team` widens one call to the whole project.

### Shape the idea

Let the next uncertainty choose the investigation. Compare alternatives against the same outcome and constraints. Save reusable evidence as facts, unresolved choices as Questions, and settled choices as Decisions, each with its actual origin. Keep local progress in the Task log. Bring the user your findings, recommendation and the remaining question the evidence cannot answer.

For sustained research, start an investigation Task from the idea, with evidence or a discussable design as its result. For brief intake, leave the next step in the idea; Status can be quiet without an active Task. An idea or spec must state whether its direction is proposed or agreed and what establishes that agreement: a Note's `active` state means maintained, not approved or implemented.

Create a spec when expected behavior needs its own maintained document. Preserve canonical sources when importing existing material: record the useful conclusion and link to the original, keeping private evidence under the chosen sharing policy. Judge how much structure the work needs; a fixed set of documents adds maintenance without answering a question.

### Model the domain

Start from concrete scenarios and check them against the code. Explain who owns state, which changes must agree, what may happen and what information crosses contexts. Distinguish existing behavior from a proposed model. A directory or class name alone does not establish an aggregate or bounded context.

| A term defines | A model explains | Why the distinction matters |
|---|---|---|
| A candidate operation | Who approves it, what approval changes and what happens after rejection | Definitions alone cannot establish allowed behavior |
| A package within one context | How each context uses it and translates information for another | The same word need not describe the same concept |

Keep local definitions in the model; extract terms when they need independent lookup or reuse. Split models where language or responsibility differs. A context map cites those models and explains integration direction and meaning. Tag models `domain-model` and related records by context so search finds them. Cite governing Decisions from models and models from specs and Tasks; keep each ruling in one place. Write a model or a spec of several paragraphs from a file with `--body-file model.md`, or from a pipe with `--body-file -`; a document does not belong on a command line, and `edit --body-file` replaces a body without reading the record file by hand.

### Plan and execute

Make each Task a reviewable result with constraints, behavior to preserve and completion evidence. For an epic, create a hub `--from` the idea, tag it `epic`, create children `--from` the hub, and `block <hub> <child>` for each deliverable. Add child dependencies only where one result is required by another. Read `ready --for <hub>` to choose work. A ready hub still needs verification of the overall outcome.

Keep one Task in flight per person by default. Start it before work; when switching subjects, log the handoff and hold unfinished work with a reason, or hand the Task over with `edit --taken-by`. Comment with the result, evidence and next step, using `--via`. Update models and specs when their meaning changes. Supersede a Decision when its ruling changes; edit it when clarifying the same ruling. A Decision that cites another as context declares the relationship once, on `add` or later with `edit`: `--link "within decision.x"` for a rule that is part of a wider one, `--link "departs-from decision.x"` for a drift. `may-conflict` then names only the pair nobody has judged; read those records before deciding whether a conflict exists.

Close answered Questions with `--resolved-by <decision-or-task>`, or `--reason` citing a Note when knowledge answers them, then archive. A genuine review date can be set on a drift Decision with `edit --review-by`; leave it unset when none is known.

### Verify and leave a continuation

Verify the promised result before closing. If review is required, `submit` and wait for acceptance. Otherwise close with `--note <report.md>` by default: state the result, evidence and limits. Archive the Task immediately; its report travels with it. Keep reusable knowledge live. Cancel work with `--reason`; hold work that awaits something, naming what will unblock it.

Run `anb check`, address findings and recheck. When no CLI repair exists, report the obstruction. Leave unfinished work's result and next action in its log, or in the idea for brief shaping. Commit the notebook with the code by default, within the user's sharing policy and commit authorization.

## Common mistakes

| Temptation | Use instead | Reason |
|---|---|---|
| Put the whole proposal in a Task to save time | Keep the idea and create work from it | Archiving one delivery must not hide the proposal |
| Record a plausible answer as a Decision | Keep a Question until the choice is settled | Future agents treat Decisions as governing knowledge |
| Use a guide as a one-off handoff | Put the next step in the Task log or idea | Guides describe repeatable procedures |
| Turn related subjects into blockers | Cite bare ids for context; block real prerequisites | Artificial dependencies hide work that can start |
| Quote an id intended as a relationship | Cite it outside backticks | Quoted examples do not create mentions |
| Retry a refused command unchanged | Read its `try:` instruction and fill its placeholders | Refusals explain the required correction |
| Repeat `add` after an uncertain result | Inspect the notebook first | Creation without an explicit id can produce duplicates |
| Start a Task marked as another person's | Take one from `ready --untaken`, or ask the user before `edit --taken-by` | Two people working one Task learn of it from a merge conflict |
| Leave a Task in the pool when its doer is already known | `add task --mine` for the user, `--taken-by <name>` for a colleague | A Task nobody holds enters nobody's queue |
| Start work straight from Status | Read `list --type decision --kind rule` first | Status is the work; a rule is read before the work it binds |

## References

Before an unfamiliar command, read `anb <verb> --help` or [commands](references/commands.md). Read [the worked session](references/session.md) for literal replies and [refusals](references/refusals.md) when recovery is unclear. Use `--json` for programmatic reads; `list` and `ready` rows carry `by` and `taken-by` there, and Status carries the pool as `untaken.count`. Use the installed `anb-atlas` skill for a visual review.

For a named personal practice, `list --match <name> --global` finds it and `show <id> --global` reads it; guides tagged `skill` are reusable practices. Global scope holds Decisions and Notes, while Tasks and Questions stay in the project. Cite a global rule's id when a project Decision departs from it so the relationship remains visible.
