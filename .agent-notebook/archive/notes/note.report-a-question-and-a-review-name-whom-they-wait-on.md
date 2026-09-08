---
id: note.report-a-question-and-a-review-name-whom-they-wait-on
type: note
state: retired
title: Report: A Question and a review name whom they wait on
by: Maksim Yaromin
from: task.a-question-and-a-review-name-whom-they-wait-on
created: 2026-09-08
updated: 2026-09-08
---

# Report: a link is an edge, and a record names whom it waits on

Closes [issue 64](https://github.com/maksimyaromin/agent-notebook/issues/64) and [issue 79](https://github.com/maksimyaromin/agent-notebook/issues/79) in one release, under the owner's ruling given during the work: give the new relations, keep the hand-off the smallest possible, mention both in the skill and teach no workflow around them, and drop whatever the issues asked beyond that.

## Result

### A link to a record id is an edge (issue 64)

| Asked | Delivered |
|---|---|
| `show <target> --all` lists incoming links by kind | `linked-by[N]: <id> (<kind>), …` on `show`, live carriers only, by kind then id; JSON `linked-by: {count, rows: [{id, kind}]}` |
| `graph` carries the edge with the link kind | `EdgeKind::Link(kind)`: the edge runs out of the record carrying the line into the record it names, under the link's own word; JSON `v` is 4 |
| `list --for` and `graph --for` reach the records that link the target | `MembershipIndex` follows `linked_by` beside `waits_on` and `born_inside`; `--focus` and the archived-kin walk follow the same edge through `kin_of` |
| `search` matches link targets | Dropped. Git already ties the notebook's records to the code they shipped with, since the notebook is committed beside it; `--for` answers the schema question |

One home: `Record::linked_records` yields `(kind, id)` for every link whose target is a record id; every walk reads it. `EdgeKind::word` spells every edge once for both renderers, which removed the duplicate match the text and JSON renderers each carried.

### A Question and a review name whom they wait on (issue 79)

| Asked | Delivered |
|---|---|
| `add question --to`, `edit --to`, `edit --clear to` | Yes; `to` is a Task or Question field, one non-empty line, trimmed, guarded like `taken-by` |
| `submit --to`; `start` clears it | `submit --to` writes it; `submit` alone keeps a standing one. `start` does not clear it: `Record::waits_on` derives the fact from the state, so a Task taken back waits on nobody while active and on the same person when submitted again. `add task --to` is legal for the same reason: the addressee is whom the next `submit` hands it to |
| A narrowed Status shows what is addressed to me beside what is mine, each row saying which | `Record::concerns` is holder, author or addressee, and is what `--by`, `--mine`, `scope: mine` and the own-first order read. `review[N]{id,taken-by,to}` is a table like `held`; `questions` gains a `to` column |
| `list --to <name>`, `--to-me`, `--mine` covering both | `--to <name>` on `list`, `ready` and `graph`, a predicate that composes with every other flag; `--mine` covers the addressed records; `--to-me` dropped as a second spelling of one identity |
| `to` in JSON rows beside `by` and `taken-by` | Yes, through `Attribution` |
| Debt names the addressee | Dropped: a row of Debt names its record, and the record names its addressee |

The skill gains one sentence per relation. The worked session puts a doubt to Grace, hands the review to her and opens her narrowed Status on both.

## The audit the owner asked for

**Is the API consistent?** The person fields are `by`, `taken-by`, `to`; each has its reading: `--by` and `--mine` read `concerns`, `--untaken` reads the holder, `--to` reads the addressee. The relation keys are `from`, `blocked-by`, `link`; each is an edge with one word, walked by the same three reads (`show`, `graph`, `--for`). `to` is written where the wait begins (`add` for a Question, `submit` for a Task) and corrected where every field is (`edit`). Every listing's truncation hint carries `--to` like every other flag.

**Is anything excess?** Four things the issues asked for were left out, each with its reason in the Decision: `--to-me`, `start` erasing `to`, Debt naming the addressee, `--match` over link lines. The review section became a table rather than gaining a mark and a phrase, because a row with two facts is a table everywhere else on the dashboard.

**Is the data structure as flexible as it can be?** A link's kind is an open token, so any hierarchy (`schema`, `within`, `departs-from`, `part-of`) is one line and no new key. `to` is a plain field whose meaning follows the state, so a reviewer can be named ahead of the first `submit` and survives the review loop, and relaxing nothing later breaks no file. The graph contract is versioned, so a consumer built on three edge words learns of the fourth.

**Are the docs updated?** Records (the `to` row, a relations section, the lifecycle paragraph), Status (both tables, whose work, JSON), Replies (narrowing, bounds, JSON rows), the session, tasks, knowledge, atlas and customization guides, the commands reference regenerated, both skills.

**Is the solution clean and efficient?** Every fold stays one pass over records already read; the view adds an envelope scan beside the body scan it had. Each rule has one home (`linked_records`, `addressee`, `waits_on`, `concerns`, `EdgeKind::word`).

## Evidence

- `./scripts/check.sh` green: fmt, clippy, every test, doctests, rustdoc, the skill and the reference pages without drift. `pnpm docs:check` green: 20 pages reachable, every link resolves.
- Twelve new tests (Core: the `linked-by` block, the link edge and its precedence over a mention, the scope and the focus through a link, the addressed draft, `submit --to` through the loop, `edit --to`, the listing narrowed by addressee, the addressed dashboard; shell: `show` and `graph` with a link, the addressed dashboard on both renderings, `submit --to` and `add --to` with `list --to`, the `to` orphan and its eraser). Each proven red by cutting the behaviour it specifies.
- The existing expectations that encoded the old review line, the old questions table and the graph contract's version were rewritten to the new shapes; the `submit` call sites and the `Attribution` literals in the tests gained the new argument and field.
- The repository's own notebook exercises the link edge: the new Decision is linked `within` the identity ruling, and `show` and `list --for` read it back.

## Smoke check

One Sonnet pass over the whole diff, with the engineering instruction loaded, every claim replayed on the built binary in scratch notebooks under two identities and `scope: mine`, and the tests spot-checked red by breaking `Record::concerns` and `EdgeKind::word`. Every claim held, including the owner's exclusions. Two findings, both holes a malformed `--link` could open, both fixed: a link kind spelled `waits`, `born` or `mentions` drew an edge indistinguishable from the tool's own and backwards to it, so `guard_link` now refuses the three words the graph draws itself, owned as `EdgeKind::DRAWN_WORDS`; and a record could link itself into a self-loop that inflated its degree while `show` hid it, so `edit` refuses a link naming its own record, as it refuses an origin naming it, and `Record::linked_records` draws no edge from a hand-written self-link. One test covers both, proven red with each guard cut.
