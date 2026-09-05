---
id: task.why-rust-dossier-release-gate
type: task
state: closed
title: Why-Rust dossier (release gate)
by: Maksim Yaromin
via: claude-code
tags: gate
link: note note.report-why-rust-dossier-release-gate
blocked-by: task.milestone-self-host-switch
blocked-by: task.milestone-cli-complete
blocked-by: task.skills
created: 2026-08-29
updated: 2026-09-05
closed: 2026-09-05
---

Standing ticket: collect candidate answers to 'why Rust'. Closes by recording 3-5 clear, unambiguous answers before first public release. 'For fun' / 'maintainer's call' is not an admissible answer — answers must stand on technical or product merit.
- 2026-09-05 Maksim Yaromin: Maintainer ruling 2026-09-05: the agent drafts the three to five answers into the task report from what the codebase shows; the maintainer accepts them at the final review after the marathon.
- 2026-09-05 Maksim Yaromin: Five answers drafted from measurements of the repository (1.7 MB static binary, sub-millisecond start, 4 runtime deps, unsafe forbidden, 666 tests): cost per session, invariants held by the compiler, byte-exact round-trip as the default, one Core for many hosts including wasm32, an auditable surface. Two are shared with Go, three are Rust's own; the report says which. Maintainer accepts or reopens at the final review.
