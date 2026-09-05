---
id: task.write-paths-and-check-read-the-global-root
type: task
state: closed
title: The write paths and check read the user's notebook too
by: Maksim Yaromin
from: task.improvements
tags: cli
link: note note.report-the-write-paths-and-check-read
priority: 3
created: 2026-09-02
updated: 2026-09-05
closed: 2026-09-05
---

Status reads the user's notebook behind the project's and names a shadow with both ids. The write paths (add, ask, decide, note, comment, edit, close with a report) probe a body's citations against the project alone, so recording the very rule that shadows a global one answers dangling-mention while Status answers shadow: one fact, two names, one of them false. check verifies every link target that parses as an id against the project alone, so a link to a global id fails the gate on an id that exists. Give both the same second root Status has: one resolver answers for every surface, the nudge names the shadow the moment the rule is written, and check stops failing on a true edge. The second root stays optional and read-only: an unreadable home drops the hint and never fails a write or the gate. Once a link to a global id is legal, shadow detection reads links as well as prose, or the docs say which edge surfaces a shadow. Every reply that can name a dangling mention gets a test posing a global id. Answers question.should-a-write-time-nudge-see-the-user-s.
- 2026-09-05 Maksim Yaromin: Done 2026-09-05: the user's notebook is a field of the Notebook handle (with_user), read by the write-time nudge, by check for link targets, and by the shadow heuristic from links as from prose; envelope edges stay the project's own. Review: two must-fix — a rule cited in prose and linked printed its shadow pair twice (deduped, test added), and the Host field's doc still named Status alone (widened). gate green.
