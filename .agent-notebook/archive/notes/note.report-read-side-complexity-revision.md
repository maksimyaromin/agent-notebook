---
id: note.report-read-side-complexity-revision
type: note
state: retired
title: Report: Status is where the work continues; reads narrow by kind, tag and identity; the read side gets a complexity revision
by: Maksim Yaromin
from: task.read-side-complexity-revision
created: 2026-09-08
updated: 2026-09-08
---

# Report: Status is where the work continues; reads narrow by kind, tag and identity

Closes [issue 71](https://github.com/maksimyaromin/agent-notebook/issues/71), which supersedes issue 70: a team on one notebook needs Status to be the work, one listing surface narrowed the same way everywhere, and a complexity revision of the read side before another flag is added.

## The inventory, and what became of it

Before: eight read verbs, twenty-two flags, five row printers. After: seven verbs (`status`, `ready`, `list`, `show`, `check`, `debt`, `graph`), one shared narrowing documented once, three row printers (the listing row, the ready row, the graph node).

| Duplicate the issue named | Ruling | Reason |
|---|---|---|
| `graph --ready` repeats `ready` | deleted | a ready Task waits on nothing live, so a graph of ready Tasks alone is a listing drawn as tiles; every node still carries `ready` for a page to key on |
| `graph --type` is the type filter `list` lacks | kept on `graph`, added to `list` | both read the one Filter |
| `overview` is `list` grouped | deleted | `list --all` is the page, `list --type <t>` a section of it; the archive counts it printed answered no question a listing does not |
| `search` is `list` with a substring | deleted | `list --match <text>` is one more predicate; `--archive` reaches history as `search` did by default |
| `--mine` is `--by` with a name filled in | kept as that spelling | a hint that lifts a bound has to carry a name that runs for whoever types it |

## The one narrowing

`Filter { types, kinds, tags, hub, by, text, archive }` lives in the Core's request vocabulary. `list` and `graph` take all of it; `ready` takes what a queue of live Tasks can answer (`for`, `tag`, `text`, and whose); `status` takes `by`, `mine` and `team`. Each flag is a predicate over the same notebook, two flags ask for the intersection, and a filter narrows what is shown and never what is read: a row is blocked, excluded and resolved against every record first, so no narrowing frees a blocked Task or changes a hub's count. The Core refuses a filter no record can answer (a kind no type allows, a malformed tag, an empty text) rather than answering an empty listing that reads as a notebook holding nothing of the kind.

Whose records a read answers with: `--by <name>` names one identity; `--mine` is `--by` with the identity the host acts as; `--team` is everyone's; when a call names none of the three, the new config key `scope` decides (`team` by default, `mine` for every read narrowed to the caller). A host that knows nobody is refused `--mine` and `scope: mine` with the fix named and `--team` offered; the hook fails soft to silence, as it does for every refusal.

In the shell, a `Narrowing` clap struct (with `Whose` inside it) is flattened into `ready`, `list` and `graph`, an `Extent` struct (`--type`, `--kind`, `--archive`) into `list` and `graph` only, and `Whose` alone into `status`, all under one help heading. `anb <verb> --help` prints the narrowing under "Narrowing", and the commands reference renders the table once at its top with each verb's section naming the flags it takes. One `narrowing_flags` spells a Filter back into a command line, so the truncation hint of a listing and of a graph carries every flag it was asked with, shell-quoted.

## Status

The subject is the work: `active` with the first Task's last log line, `review`, `held` (now with a `taken-by` column), `ready`, `questions` (id, age, who asked, title), and `debt: N — anb debt`. Under the team scope every section leads with the reader's own and marks another person's line with their name; under `--mine` or `scope: mine` the sections hold only that identity's records and a `by: <name> — anb status --team` line says so, with every hint carrying the same `--by`. Rules and epics have no section, and knowledge never opens the gate: a notebook of rules alone is quiet, which reverses the ruling shipped for issue 66 on the issue's explicit ask. The budget ladder is now: ready rows, the questions to a count, the log with review and held to counts, the floor.

`anb debt` is the read of its own: every signal in the clock table's order, bounded like every listing, the same rows in JSON as `{count, debt}`. The Status JSON carries `by` when narrowed, `questions`, and `debt` as `{count}`, so a consumer reading `debt.count` keeps working.

Epics: `ready --for <hub>` is an epic's queue, `list --for <hub> --archive` its whole membership, and a hub's graph node carries `closed/total` and `next` as before. The dashboard no longer opens any archived file.

## Evidence

- `./scripts/check.sh` green: fmt, clippy `-D warnings` over every target, the Core suite (386 in the notebook suite, 12 archive-read cases, 5 property cases), the CLI suite (169), the filesystem and process suites, doctests, rustdoc, the skill and the reference pages regenerated without drift.
- `pnpm docs:check` green: 20 pages reachable, every link resolves.
- New behaviour tests, thirteen of them proven red by an inverted expectation after the fact and the rest red during the work: the narrowing by type, kind, tag, text and their intersection; a kind no type allows and a malformed tag refused; the archive joining a listing only when asked for; the queue narrowed by the same filter; a narrowing by identity leaving a blocked Task blocked; the questions section's order and columns; review and held leading with the reader's own; the narrowed dashboard's `by:` line, hints and quiet form; the `scope` key read, refused without an identity, widened by `--team`; the hook reading the key; the debt verb in text and JSON; the graph slice carrying `match`, `tag` and no `ready`; the ladder sweep over every ceiling from 400 down.
- The worked session in the skill now shows `list --type decision --kind rule`, `status --mine`, and `list --match fence --archive`; the Status example moved before the Question closes so it shows a `questions` row.
- Book: status, replies, records (the `scope` key), session, knowledge, tasks, atlas, ideas, customization, your-own-notebook, what-it-is, quickstart, development, index; README and AGENTS.md; the atlas skill and its drawing reference (graph JSON contract 2 to 3).

## Smoke check

A Sonnet 5 agent loaded the engineering instruction, re-read the whole diff, ran the gate and the docs check, and exercised every narrowing against scratch notebooks. No behaviour defect; four text findings, all fixed: a fixture comment in the property tests claimed hubs reach the queue (they stay blocked by their children); the skill's global-practice sentence still said "search"; the `anb debt` renderer's comment narrated the dashboard's former header; the `scope: mine` refusal read `scope: mine: no identity to match` and now reads `scope: mine needs an identity; set git user.name or ANB_BY, or pass --team`.

## Decisions

- `decision.one-narrowing-for-every-read` (shape) records the Filter and the rulings per duplicate. A flag means the same on every verb that takes it, and a verb takes only the flags it can answer: `ready` refuses `--type`, `--kind` and `--archive` on the command line, since a queue of live Tasks could only answer them with nothing more.
- The hook stays silent on a `scope: mine` notebook without an identity, as on every refusal: the fail-soft has one rule, and the refusal is one `anb status` away.

## Limits

- The `held` and `review` sections name a person by `taken-by`, else `by`; a Task nobody is named on is nobody's in every section.
- `decision.a-dashboard-reads-archived-lineage-only` still stands; with no epic section the dashboard reads no lineage at all, which is inside its rule.
- Graph JSON consumers built against contract 2 read `slice.ready` and `v: 2`; both changed with the version.
