# Changelog

## [0.1.0] - 2026-09-05

### Added

- Scaffold the Rust workspace — anb-core behind the Storage seam + anb CLI
- Grammar layer — total lossless parse, render, normalize, named findings, corpus
- Record model — typed lifecycles, write-time invariants, idempotent mutations
- Dependency graph — cycle rejection, unblocked on close, ready queue
- Status + Budget — gated dashboard, full Debt, degradation ladder
- CLI task cycle — fs adapter, output contract, recovery payloads, Status surface
- CLI knowledge cycle — decide/note/ask/answer/retire, conflict nudge
- Quotation rule — code spans and fences quote ids, mutating verbs nudge on dangling mentions
- Check, archive, edit, search, overview — the CLI's maintenance surface
- Name the state-vs-residence split, in both directions
- Close --note ingests the report as the Note it links
- Expunge removes a record born by mistake, guarded by its inbound edges
- The notebook's location is the user's to choose
- Epics are hub Tasks, with the queries and the dashboard to work them
- Status names the proofs the world no longer holds
- Check names the repair, view bounds a long body, and the storage seam stops at the root
- The user's notebook, and the verbs that have nothing to do there
- The rules behind a project, named beside the ones that shadow them
- The notebook as a graph — data by default, a picture on request
- The graph is data, and the drawing belongs to whoever asked for it
- The gate runs on GitHub too — CI on every PR and push to main
- The one corruption the tool can name is one it can undo ([#4](https://github.com/maksimyaromin/agent-notebook/pull/4))
- The CLI speaks one plain word per concept, and a Task ends without work by reason ([#7](https://github.com/maksimyaromin/agent-notebook/pull/7))
- The write paths and check read the user's notebook too ([#8](https://github.com/maksimyaromin/agent-notebook/pull/8))
- Setup wires the agents' session start to the notebook ([#10](https://github.com/maksimyaromin/agent-notebook/pull/10))
- The skill is rendered by the binary, installed by setup, and diffed in CI ([#11](https://github.com/maksimyaromin/agent-notebook/pull/11))
- The atlas skill draws the notebook and turns the reader's comments into commands ([#13](https://github.com/maksimyaromin/agent-notebook/pull/13))
- Npm distribution, the shim package and one package per platform ([#17](https://github.com/maksimyaromin/agent-notebook/pull/17))
- A tag publishes a GitHub release with the binaries and the changelog entry as its notes ([#27](https://github.com/maksimyaromin/agent-notebook/pull/27))

### Fixed

- Mint ids that cut at a word boundary
- Read a report proof's path from the project, not the notebook directory
- A repair is progress, not perfection — and only where a verb can reach
- What two reviews found — a live record answers for its own id, the gate's first hop is one hop, and the claims match the code
- The protocol the type system now keeps, and a line a record could forge
- Seven defects a correctness audit reproduced, none of them cheap
- What the intake review found — one home for the archive prefix, and four comments that outran their subject
- An interrupted move is recognised by identity, never by bytes
- A browser that never opens the map is ended with the run that started it
- A held Task waits in its own Status section, never as the active one ([#9](https://github.com/maksimyaromin/agent-notebook/pull/9))
- The npm packages stay out of the pnpm workspace, so the frozen install holds ([#18](https://github.com/maksimyaromin/agent-notebook/pull/18))
- The Pages workflow builds and stops until the Cloudflare secrets exist ([#19](https://github.com/maksimyaromin/agent-notebook/pull/19))
- A count and its noun agree in number ([#23](https://github.com/maksimyaromin/agent-notebook/pull/23))

### Changed

- An audit pass — the walks stop overflowing, the dashboard stops reading history it has no epic for, and the suite guards what it claimed
- Obligations the type system keeps, and one path to every name

### Documentation

- The README a stranger reads first, and the MIT license ([#15](https://github.com/maksimyaromin/agent-notebook/pull/15))
- The documentation site, built from docs/ and deployed to Cloudflare Pages ([#16](https://github.com/maksimyaromin/agent-notebook/pull/16))
- AGENTS.md says what a stranger's agent needs, and nothing that lives outside the repository ([#20](https://github.com/maksimyaromin/agent-notebook/pull/20))
- Public text describes the tool and its scripts, never a maintainer's setup ([#21](https://github.com/maksimyaromin/agent-notebook/pull/21))
- The skills name mistakes the tool makes easy, and the atlas page stays out of the map's way ([#22](https://github.com/maksimyaromin/agent-notebook/pull/22))
- The README and the book rewritten for engineers, with a customization guide ([#26](https://github.com/maksimyaromin/agent-notebook/pull/26))

[0.1.0]: https://github.com/maksimyaromin/agent-notebook/releases/tag/v0.1.0

