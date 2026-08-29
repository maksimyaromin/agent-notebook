---
id: task.scan-code-span-quotation-write-time-unre
type: task
state: review
title: Scan: code-span quotation + write-time unresolved-id warning
by: Maksim Yaromin
via: claude-code
from: decision.backticked-id-is-a-quotation
priority: 1
created: 2026-08-29
updated: 2026-08-29
---

Two halves of one rule. (1) The mention scan skips markdown code spans: an id inside backticks is a quotation and creates no mention edge; a bare id stays a reference. (2) The write-time nudge: every mutating command that accepts body or comment text warns in its own output when that text introduces a bare id that does not resolve — suggest backticking to quote, or creating the record. No rejection: forward references stay legal. Acceptance: the four migrated quotation sites (backticked by hand) stop producing dangling-mention debt; a bare unknown id in a fresh comment produces the warning; a backticked unknown id produces neither warning nor debt.
- 2026-08-29 claude-code: Plan settled: (1) mention scan skips backtick code spans via equal-length run pairing, unpaired runs stay literal; (2) create/comment/drop replies gain dangling_mentions computed against live+archive, rendered as a dangling-mention line with the backtick-or-create hint in both formats; drop_question grows a Dropped reply struct.
- 2026-08-29 claude-code: Implemented and green: scan skips code spans (equal-length backtick-run pairing, unpaired runs literal); create/comment/drop replies carry dangling_mentions with the backtick-or-create hint in text and JSON. 16 new tests, all proven red by expectation flips; check.sh exit 0. Field-verified: the four migrated quotation sites no longer produce debt, a scratch notebook shows the warning on bare unknown ids and silence on backticked ones. Next: Opus 5 review.
- 2026-08-29 claude-code: Opus 5 review: 1 defect (span pairing crossed lines — a stray backtick in one entry unquoted later entries; fixed by bounding span closers to the line), 1 should-fix (fence rule was accidental — now real: line-leading 3+ run, closes on a run at least as long, unclosed swallows to end), 3 missing tests added (drop-replay nudge, quoted id silences may-conflict, multi-byte neighbors), 4 cleanups applied (Answered split into Routed/Dropped so the empty-nudge invariant is unconstructible; duplicate doc deleted; doc trimmed; commented_lines extracted). All 6 new tests flip-proven red; check.sh exit 0; defect repro re-run clean.
