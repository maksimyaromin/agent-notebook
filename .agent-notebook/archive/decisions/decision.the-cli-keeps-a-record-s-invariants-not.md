---
id: decision.the-cli-keeps-a-record-s-invariants-not
type: decision
state: superseded
kind: rule
title: The CLI keeps a record's invariants, not a team's workflow
by: Maksim Yaromin
via: claude-code
from: task.status-a-held-task-is-not-in-flight
superseded-by: decision.session-continuation
created: 2026-09-05
updated: 2026-09-12
---

start does not refuse while another Task is active, and no verb will: one Task in flight at a time is a working protocol, taught by the skill and shown by Status, not a rule of the record model. The Core enforces what makes a notebook readable by any team — states, edges, references, the close outcome — and leaves how many things a team works on at once to the team. A notebook read by several agents or several people is the ordinary case, and a tool that refused the second start would be choosing their process for them. Recorded 2026-09-05 while Status learned to keep a held Task out of its active lines: what a session resumes from is the active Task that is not on hold, and a held one waits in its own section with its reason.
