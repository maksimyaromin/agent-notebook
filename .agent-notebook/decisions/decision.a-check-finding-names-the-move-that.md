---
id: decision.a-check-finding-names-the-move-that
type: decision
state: active
kind: rule
title: A check finding names the move that erases it
by: Maksim Yaromin
via: claude-code
from: question.should-a-check-finding-carry-the-command
tags: cli
created: 2026-08-30
updated: 2026-08-30
---

A check row carries a repair: the literal command that erases the finding, or `-` when the notebook has no move for it. The repair is read off the line the finding sits on — a blocked-by line is erased by unblock, a line carrying an optional field the record can lose by edit --clear, a settled record still in the working set by archive — so one rule covers a bad value, a dangling reference, a duplicated key and a cycle alike. A finding on a line no verb writes carries none, and the CLI gains no eraser for a machine-written field: kind, by, via, link, supersedes, superseded-by, routed-to, updated and closed are written by verbs and never by hand, so a hand-broken one is a hand repair.

The repair is not a function of the code, which is why it is not one line per report: dangling-ref on blocked-by is unblock, on from is edit --clear from, and on supersedes is nothing; bad-value on a priority line is a clear and on a state line is nothing. Reading it off the line is what makes it exact, and what keeps it from going stale — it is derived at check time from the same list the edit guard admits.

A record broken in more ways than one refuses its own repair until the findings no verb reaches are hand-repaired; the refusal names them, so the agent learns what stands in the way rather than being told nothing.
