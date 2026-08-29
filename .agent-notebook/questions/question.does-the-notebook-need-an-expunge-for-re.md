---
id: question.does-the-notebook-need-an-expunge-for-re
type: question
state: open
title: Does the notebook need an expunge for records born by mistake?
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
created: 2026-08-29
updated: 2026-08-29
---

retire and supersession end a record that was once true; archive will move records with their history. A record created in error is a third case: it should leave no trace at all, and no verb covers it. Today the only path is deleting the file by hand — legal, the markdown is the source of truth, but it bypasses the CLI, checks nothing derived (mentions, origins, edges pointing at the id), and contradicts the mutate-only-through-the-CLI discipline. Candidates: an expunge verb that refuses when anything references the id, or a documented ruling that mistakes are repaired by hand plus git and check owns the cleanup.
