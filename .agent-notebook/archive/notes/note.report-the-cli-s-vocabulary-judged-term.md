---
id: note.report-the-cli-s-vocabulary-judged-term
type: note
state: retired
title: Report: The CLI's vocabulary, judged term by term
by: Maksim Yaromin
from: task.the-cli-s-vocabulary-judged-term-by-term
created: 2026-09-05
updated: 2026-09-05
---

# The CLI's vocabulary, judged term by term (2026-09-05)

Report for task.the-cli-s-vocabulary-judged-term-by-term. The maintainer asked for serious analytical work: every word the CLI speaks, at every level, judged on stated criteria, with the reason and the rejected alternatives per verdict. Two words were refused before this report existed — `drop` (the Question's reasoned ending, and the Task ending's first draft) and `withdraw` (the second draft) — which set the bar: no word chosen on the fly.

## 1. Criteria

1. **An engineering product for engineers.** The plainest word an engineer already uses for the concept, never a term borrowed from another field.
2. **Unification.** One word for one concept across every level — verb, flag, state value, envelope key, reply label — and the fewest distinct words in the whole vocabulary.
3. **No over-complication.** A made-up synonym is worse than an apt everyday word; a new word needs a concept no existing word covers.
4. **The maintainer's ear and eye.** Rounded, smooth words without ragged letters. `agent-notebook` is the model; `withdraw` the counterexample.

Criterion 4 has a measurable core. The bouba/kiki effect (Ramachandran & Hubbard 2001; Ćwiek et al. 2022, N=917 across 25 languages: "bouba" read as round in 22 of 25, "kiki" as spiky in only 11 of 25) and the phonaesthetics literature it rests on classify phonemes: **round** are the sonorants and voiced consonants and back vowels — l, m, n, b, d, g, a, o, u — **spiky** are the voiceless stops and the high front vowel — p, t, k, i. On the page the ragged letters are the angular ones: w, v, k, x, z, y. So a word scores well when it leans on l m n b d g a o u and carries at most one of p t k i w v x y z. `agent-notebook` passes (one t, one k, the rest round); `withdraw` fails (w, i, t, w).

## 2. Method

- Inventory first: every verb, flag, state, key, kind, code and reply label at HEAD.
- A systematic sweep: 19 practitioner tools (git, gh, GitHub Issues and Discussions, GitLab, Jira, Linear, Shortcut, Azure Boards, Taskwarrior, todo.txt, beads, Backlog.md, Gerrit, Phabricator, Bugzilla, Stack Overflow, adr-tools, MADR, PEP 1, IETF) and three theory areas — CLI design guidelines, LLM tool-naming ablations, sound symbolism — every quote verified against its page.
- Verdicts here, composed against the four criteria with the sweep as evidence.

### What the sweep says that changes verdicts

- **Near-universal words** exist for a handful of concepts and are kept where we already use them: `reopen` (the most consistent word in the whole sweep), `close`, `blocks / blocked by`, `ready` (every graph-based agent tool: beads, Taskwarrior), `archive`, `delete`, `edit`, `search`, `status`, `comment`, `label`/`tag` (split down the middle).
- **"Ended without doing the work" has no stable word anywhere.** Twenty-plus spellings: `not planned` / `not_planned` (GitHub, two spellings in one product), `Won't do`, `wontfix`/`WONTFIX`, `Canceled` (Linear, American only — no tool ships the British spelling), `Abandoned` (Gerrit alone), `Dismissed` (beads alone), `Removed`, `Deferred`, `deleted`, a dozen Bugzilla instance-specific resolutions. The one engineer-standard *shape* is GitHub's: the ending is a **flag on close** — `gh issue close --reason …` — not a verb of its own.
- **A question's settlement** is called "answer" by GitHub Discussions ("Mark as answer") and Stack Overflow ("accept"), and "resolved" by every tracker that closes an item against a change (GitLab and Linear closing keywords `Resolves`, Phorge "Resolved", GitHub Discussions' close reason "Resolved").
- **CLI guidelines converge** (clig.dev, GNU standards, Heroku, the GitHub CLI primer): the same verb for the same action across every object; modifiers as flags, never as separate commands (`gh pr review --approve`, not `gh pr approve`); no near-synonyms (`update` beside `upgrade`); language that cannot be misconstrued (`gh pr open` is the primer's own bad example).
- **Naming matters less than descriptions for agents.** Four ablations (ToolSandbox, Live API-Bench, GradleFixer, Canary Tools): scrambling a tool's *name* while its description stays accurate costs frontier models 0–0.5 points, sometimes nothing; thinning or misleading the *description* costs 10 points to total collapse. So the plainness of a verb buys understanding for the human reading a file and a help screen; for the agent it is the help text and the reply that carry the weight — which is why every rename here comes with its help line, and why no verb is chosen for cleverness.

## 3. Verdicts

Legend: **keep** — the word stays; **rename** — the word changes, old → new; **fold** — the concept moves under another word and the old word disappears; **new** — a word the vocabulary did not have.

### 3.1 Verbs — 28 become 23

| Term | Verdict | Why | Rejected |
|---|---|---|---|
| `add` | keep, widened: `add <type> "<title>"` for every record type | The GitHub CLI primer's rule "any create command should behave the same" and clig.dev's "consistent verbs across objects" both say one creation verb; `add` is Taskwarrior's, todo.txt's and Azure's word and reads as one round syllable. The type is the first positional — `task`, `decision`, `note`, `question` — the very words the format already uses, so no word is added. | The type as the verb (`anb task "…"`): no tool in the sweep does it, and the primer names noun-as-command as the misconstruable shape. `create`/`new`: gh's and beads' word, equally plain, longer, spikier (c, t / n-e-w). A per-type flag (`--type decision`, beads' `bd create --type`): the type is never optional, so a flag hides a positional. |
| `decide`, `ask` | fold into `add decision`, `add question` | Three creation verbs for one action fail criterion 2 and the primer's rule. Their charm is real; their cost is three words an agent must learn for one move. | Keeping them as aliases: a second spelling of one action is the near-synonym clig.dev warns against. |
| `note` | keep as the type word; creation is `add note` | — | — |
| `start` | keep; now also `review → active` | The one verb every tool that has the concept uses (Taskwarrior; Jira "Start progress"). Taking work back from review is starting it again; a separate verb for that exists in no tool in the sweep. | `begin` (rounder, used nowhere); `claim` (beads — ownership, not state). |
| `submit` | keep | "Submit for review" is the everyday phrase; the state it reaches is `review`. The only collision is Gerrit's `submit` = merge, one tool against the common tongue. | `review` as a verb: reads as "show me the review" — the misconstruable shape; `ready`: our dispatch queue's word (beads' sense), a collision inside our own vocabulary. |
| `close` | keep; gains `--reason` and `--resolved-by` | Universal (gh, GitLab, beads). It now ends Tasks and Questions alike — one verb for one action across objects. | `done` (Taskwarrior): equally plain, but `close` already pairs with `reopen` and with the state `closed`. |
| `return` | fold into `start` | No tool in the sweep has a "return from review" verb; the move is "back into work", which `start` says. `return` also carries a programmer's reflex meaning. | Keeping it: a verb for a move `start` already names. |
| `reopen` | keep | The single most consistent word in the sweep. | — |
| `hold`, `unhold` | keep | "On hold" is the everyday phrase; our prior art; `un-` mirrors `unblock`. The mechanism the sweep found under five different words (`waiting`, `defer`, `Snoozed`, `Blocked` flag, `wip`) has no consensus, so the plainest English wins. | `pause`/`resume` (rounder, universal in media): break the `x`/`unx` pattern and would rename the `hold` field. `defer` (beads): "deferred" reads as a decision about priority, not a pause. |
| `block`, `unblock` | keep | "Blocks / blocked by" is a near-universal pair (GitHub, GitLab, Jira, Linear, Taskwarrior, beads; Phabricator renamed to it "to reduce ambiguity"). | — |
| `comment` | keep | gh, beads, Phabricator's own word for one entry. The trail it appends to stays "the log" in prose and in Status — one entry, one trail, two things. | `log` as a verb: `git log` reads. `annotate` (Taskwarrior): a rarer word. |
| `answer` | fold into `close --resolved-by <id>` and `close --reason "<why>"` | A Question ends the way a Task does: `close`, carrying either the record that resolved it or the reason it closed without one. One verb for one action across objects; the maintainer's own reservation about `answer` needs no word to replace it. | `resolve` as a verb: Azure Boards and Bugzilla use "Resolved" for an intermediate state; as a verb it would be a second ending verb beside `close`. |
| `retire` | keep | A retired Decision or Note stops binding entirely; "deprecated" (MADR, every API) means "still works, stop using it" — the wrong promise. `retire` is what engineers say of a service or a flag. | `deprecate` (misleads on force), `obsolete` (IETF: always names a successor), `close` (English refuses "close a decision"). |
| `ready` | keep | The word of every graph-based agent tool for the blocker-aware queue. | `next` (Taskwarrior's other report), `todo` (a static column in SaaS trackers). |
| `list` | keep | Universal. | `ls` alias: clig.dev warns against arbitrary abbreviations. |
| `view` | rename → `show` | `git show`, `bd show`, Bugzilla "Show Bug"; gh's `view` is the minority. One ragged letter instead of two. | `get` (REST vocabulary, not a CLI verb), `info` (Taskwarrior). |
| `status` | keep | git, gh, beads. | `summary`, `home`. |
| `check` | keep | The sweep found five verbs for five scopes (`fsck`, `doctor`, `diagnostics`, "Sanity Check", "consistency"); `check` is the plain generic every engineer types (`cargo check`). | `doctor` (beads, Backlog.md): cute jargon. |
| `archive`, `restore` | keep | Jira's exact pair; `restore` is also git's and Gerrit's word. | `unarchive` (gh): a coinage where `restore` exists. |
| `expunge` | rename → `delete` | Universal (gh, beads, Jira, todo.txt); "expunge" is records-and-law vocabulary. The verb's safeguard — it refuses while anything cites the record — lives in its help line and its refusal, not in an unusual word. | `remove`/`rm` (git's word for tracked files), `purge` (Taskwarrior: the second stage after a soft delete — we have no soft stage). |
| `edit` | keep | gh, Jira, Linear. | `update` (beads keeps both for two mechanisms; we have one). |
| `search` | keep | gh, beads, Backlog.md. | `find`, `grep`, `query`. |
| `graph` | keep | The engineering word for what it returns. | — |
| `overview` | keep | Backlog.md's word for the same page. | `summary`, `dashboard` (a UI word). |

### 3.2 Flags

| Term | Verdict | Why | Rejected |
|---|---|---|---|
| `--reason "<why>"` | keep on `hold`; **new on `close`** | The one flag for "why": why a Task pauses, why a Task or Question closes without work or without a record. GitHub's `gh issue close --reason` is the engineer-standard shape for the ending the sweep found under twenty words. Round: r-e-a-s-o-n. On `close` it is exclusive with every proof, as two proofs already are. | `--drop`, `--withdraw` (refused), `--cancel` (spelling splits Canceled/Cancelled; no British form ships anywhere), `--abandon` (Gerrit alone; a b a n d o n is the roundest candidate, but a new word for a concept `reason` already covers), `--skip`, `--dismiss` (beads alone). |
| `--resolved-by <id>` | **new on `close`** (Question) | The record a Question closed into. "Resolves" is what GitLab, Linear and GitHub say when a change settles an item; `-by` with a record value follows `superseded-by` and `blocked-by`. | `--to` (says nothing about the relation), `--answer` (apt as a noun, but the maintainer's reservation and `resolved-by` covers it), `--routed-to` (network vocabulary). |
| `--drop` (answer) | fold into `close --reason` | — | — |
| `--to` (answer) | fold into `close --resolved-by` | — | — |
| `--note`, `--pr`, `--sha`, `--report`, `--no-proof` | keep | prior art; each names the kind of evidence. | — |
| `--priority`, `--kind`, `--supersedes` | keep, now all on `add`; the Core refuses a flag foreign to the type | One creation command carries the union; a `--priority` on a decision is refused by name, which is how every domain flag already works here. | Per-type subcommands. |
| `--from`, `--tag`, `--untag`, `--link`, `--body`, `--by`, `--via`, `--id` | keep | Plain; `tag` is split with `label` in the sweep (GitHub, GitLab, Jira, Linear, beads say label; Taskwarrior, Azure, Bugzilla say tag or keywords). `tag` stays: short, round, and the key `tags` is in every record. | `--label`/`--unlabel`: "unlabel" is a coinage. |
| `--until <DATE>` | keep | — | — |
| `--for <ID>`, `--all`, `--budget`, `--hook`, `--focus`, `--depth`, `--archive`, `--full`, `--ready` | keep | — | — |
| `--type <KIND>` (graph) | keep the flag; value name → `TYPE` | The flag names types; calling its value a kind contradicts the format's own word for `kind`. | — |
| `--title`, `--clear <FIELD>`, `--review-by <DATE>` | keep | `review-by` reads as "review this by then"; the collision with the `review` state is in the eye only. | `due` (every tracker's word, but a deadline, not a resurfacing date), `revisit-by` (v). |
| `--json`, `--notebook`, `--global` | keep | — | — |

### 3.3 The file format

| Term | Verdict | Why | Migration |
|---|---|---|---|
| Question states `open | routed | dropped` | rename → `open | closed` | The same two words as a Task's ends; the outcome lives in the envelope: `resolved-by` or `reason`. Criterion 2 at the level that binds every notebook. | Every `routed` becomes `closed` keeping its pointer under the new key; every `dropped` becomes `closed` with `reason` filled from the body line the drop wrote, and that line removed. |
| `routed-to` | rename → `resolved-by` | As the flag. | Key renamed in place, same value. |
| `reason` | **new** (Task, Question; only with `state: closed`) | The why of a close without work or record, as a field rather than a body line, because the body is opaque to the parser and this fact is one `check` verifies: a closed Question carries `resolved-by` or `reason` or it is a `missing-field` finding. | — |
| Task states, Decision states, Note states | keep | — | — |
| Kinds `rule | shape | drift`, `fact | term | guide` | keep | The notebook's own concepts, defined in the record model. | — |
| All other keys | keep | — | — |
| Body line `Dropped <date>: <reason>` | fold into `reason` | One fact, one home. | Removed by the migration. |

### 3.4 Replies, Status, Check

| Term | Verdict | Why |
|---|---|---|
| `in-flight:` | rename → `active:` | The state word itself; "in flight" is an aviation metaphor. |
| `rules[N]` | keep | The section lists only the Decisions of kind `rule` — the standing constraints — so the kind word is the exact word; `decisions[` would promise the shapes and drifts it leaves out. |
| `review-wait` | rename → `review-stale` | It fires when a Task has sat in review too long — the same clock shape as `task-stale`. |
| `undeclared-pair` | rename → `may-conflict` | The `decide` nudge already prints `may-conflict[` for the same fact. |
| `hold-quiet` | rename → `hold-stale` | Parallel to `task-stale`, `review-stale`. |
| `broken-routing` | fold into `missing-field` and `dangling-ref` | A closed Question with neither `resolved-by` nor `reason` is a missing field; a `resolved-by` naming nothing is a dangling reference — two findings the vocabulary already has. |
| `dep-cycle` | rename → `block-cycle` | The edge is written by `block`; `origin-cycle` is its sibling. |
| `routed-to:` reply line | rename → `resolved-by:` | — |
| Every other label, code and message word | keep | Plain kebab-case; the error-code set is a documented contract. |

### 3.5 The glossary

`Routing` → **Resolution**: what a closed Question names — the record that resolved it (`resolved-by`) or the reason it closed without one (`reason`). `Expunge` → **Delete**. `Withdrawal` removed. `Proof` gains one sentence: a close by reason carries no proof, because no work happened. `Question` no longer "closes only by routing": it closes like a Task, resolved or with a reason.

## 4. The vocabulary after

- **Verbs (23):** add, start, submit, close, reopen, hold, unhold, block, unblock, comment, retire, ready, list, show, status, check, archive, restore, delete, edit, search, graph, overview.
- **Creation:** `add task|decision|note|question "<title>" [--id --from --tag --link --body --by --via] [--priority] [--kind --supersedes]`.
- **Endings:** `close <task> --note|--pr|--sha|--report|--no-proof` (done), `close <task|question> --reason "<why>"` (ended without work or record, legal from `open`), `close <question> --resolved-by <id>`, `retire <decision|note>`, `reopen <task>`.
- **States:** task `open | active | review | closed`; question `open | closed`; decision `active | superseded | retired`; note `active | retired`.
- **Keys added or renamed:** `reason`, `resolved-by`.
- **Status labels:** `active:`, `rules[`, `epics[`, `ready[`, and the Debt clocks `lost-proof`, `origin-closed`, `question-age`, `task-stale`, `review-stale`, `review-due`, `may-conflict`, `hold-stale`, `shadow`.

The Task ending that task.close-drop-ends-a-task-without-work owed is `close <id> --reason "<why>"`: the reason lands in the envelope as `reason`, the close date is stamped, no proof link is written, the state is `closed`.

## 5. Pivotal choices the maintainer should glance at first

1. **One creation verb with the type as a positional** (`add decision "…"`) instead of `decide`/`ask`. The most visible change; the reasoning is the GitHub CLI primer's consistency rule.
2. **Questions close like Tasks** (`close --resolved-by`, `close --reason`; states `open | closed`) — `answer` and the `routed`/`dropped` states disappear, with a migration of this repository's own notebook.
3. **`--reason` is the one word for "why"** on both `hold` and `close`, replacing `drop`/`withdraw`.
4. **`view` → `show`, `expunge` → `delete`** — plainness over the current words.

Every verdict is a rename an afternoon reverses; none changes what the notebook can express.

## 6. Migration of this repository's notebook

The only notebook in existence is this one. The change carries a script that rewrites its 16 `routed` Questions (`state: closed`, `routed-to` → `resolved-by`) and 2 `dropped` Questions (`state: closed`, `reason:` from the `Dropped <date>:` line, the line removed), then `anb check` proves the result. Any other file still carrying the old words is a `bad-value` finding — reported, never silently read as the new word.

## 7. Implementation order

Core (state machine, envelope table, close/resolve, findings, Status labels) → CLI (commands, flags, dispatch, recovery shapes, renderers) → tests renamed and extended → migration script and this notebook → CONTEXT.md, AGENTS.md command names → the Task ending task finishes on the settled word → review → PR.

## 8. What shipped (2026-09-05)

Every verdict in section 3 is in the working tree, with one revision made while implementing: the Status section `rules[` keeps its word, because it lists only the Decisions of kind `rule` — `decisions[` would have promised the shapes and drifts it leaves out. The Decision record was corrected to say so.

- Core: the Task state machine (`Start` from review, `CloseWithReason` from every live state, `Return` gone), the envelope table (`resolved-by`, `reason`), the Question close shared by `resolve_question` and `close_with_reason`, `delete`, the finding codes, the Debt clock names and their config keys, the Status label.
- CLI: `add <type>` with the flag union, `close` with seven exclusive closings, `show`, `delete`, the recovery table keyed by the move words with the reason shape once, the renderers, the lock and scope tables.
- Tests: every renamed word swept; new behaviours guarded through the public interface; corpus cases renamed and two added (a closed Question with no outcome; a reason on an open one).
- This repository's notebook: 18 Questions migrated by script (16 `routed` → `closed` with `resolved-by`; 2 `dropped` → `closed` with `reason`, the body line removed); `check` green.
- The glossary (CONTEXT.md): Resolution and Reason and Delete replace Routing and Expunge; AGENTS.md speaks `show` and `active`.

The gate is green: fmt, clippy with warnings as errors, 144 CLI cases, 339 Core notebook cases, the corpus, the property suites, the doctests.
