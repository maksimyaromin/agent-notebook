---
id: task.shared-memory
type: task
state: closed
title: Redesign and verify shared project memory
by: Maksim Yaromin
via: codex
taken-by: Maksim Yaromin
from: note.shared-memory
created: 2026-09-12
updated: 2026-09-12
closed: 2026-09-12
---

Deliver note.shared-memory. Verify recall, capture, consolidation, private audiences, parallel sessions, team handoffs, plain Markdown reading, installation, migration and recovery after Git operations. Simplify the model and command surface, update the book and installed guidance, and verify the result through repository checks and isolated agent scenarios.
- 2026-09-12 Maksim Yaromin/codex: Implemented labelled project, personal and global recall; shared knowledge is independent of authorship. Added canonical YAML normalization and recoverable import, standard TOON output, safer filesystem writes, attributed multiline outcomes and local session focus. Removed inferred conflict warnings, implicit private reference resolution and automatic report creation. Remaining: final integration checks, audience-aware output verification and documentation review.
- 2026-09-12 Maksim Yaromin/codex: Final verification passes: 832 workspace tests and one doctest, strict clippy, rustdoc, generated skill/reference drift checks, and the complete documentation build/link gate. The standard TOON encoder passes 538 pinned official fixtures and four adversarial round trips. Six ordinary scenarios on two model tiers pass, plus four native startup/continuation trials. Concurrent next-task sessions select different eligible work and preserve another person’s assignment. Default reads remain bounded over 5000 live records. All 20 issue histories were reviewed; seven open issues were closed after verification, and a remaining review-recipient defect from a closed issue was corrected. Reproduction tools are in tools/evaluation. No commit, push or release was made.
- 2026-09-12 Maksim Yaromin/codex: Delivered shared project knowledge, project-private and cross-project practices, session-specific continuation, atomic local next-task selection, explicit joins, and native host recall. Unified outcomes and prose inputs; removed compulsory report records, inferred semantic conflicts, private shadow inference and the duplicate text renderer. Canonical YAML and standard TOON preserve readable data with one structured reply contract. Import, migration and interrupted writes preserve originals; divergent archive/restore copies are refused without data loss. The shipped method now teaches selective capture, evidence-based consolidation and independent knowledge lifetimes. Project workflow extensions survive setup upgrades without a skill fork. Source, documentation, generated instructions and reproducible evaluation runners are ready for review; all final gates pass.
