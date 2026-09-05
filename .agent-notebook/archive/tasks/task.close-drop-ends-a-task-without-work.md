---
id: task.close-drop-ends-a-task-without-work
type: task
state: closed
title: A Task ends without work, stating why
by: Maksim Yaromin
from: task.improvements
tags: cli
link: note note.report-a-task-ends-without-work-stating
priority: 3
created: 2026-09-02
updated: 2026-09-05
closed: 2026-09-05
---

{'lines': 3, 'head': '\nA Task overtaken before it was started has no honest exit: close accepts active or review only, so the ritual is a start nobody meant, then close --no-proof. Add a reasoned ending to close as the fourth way to end a Task, parallel to the Question\'s own reasoned ending: accepted from open, active and review; the reason is mandatory and lands in the log; the close date is stamped; no proof link is written. --no-proof keeps meaning done with nothing to show and keeps refusing open. State stays closed: the log carries the distinction, so epic progress, archive and reopen are untouched. The flag beside any other proof is a conflict, as two proofs are today. The word the flag and the log line carry is settled by the vocabulary audit, not here: drop and withdraw were both refused by the owner (2026-09-05). The generated skill teaches it. Answers question.can-a-task-die-without-ever-being.\n- 2026-09-05 Maksim Yaromin: The vocabulary audit settled the word (decision.the-cli-speaks-one-plain-word-per): the ending is close <id> --reason "<why>", the same --reason hold carries; the why lands in the envelope as reason, not in the log, so check can verify it. The mechanism in the working tree is renamed accordingly inside the audit\'s change.\n'}
- 2026-09-05 Maksim Yaromin: Shipped in PR #7 as close <id> --reason "<why>": envelope reason and closed date, no proof link, legal from open, active and review; question.can-a-task-die-without-ever-being is answered. Report at .tmp/docs/report-task-ends-without-work.md.
