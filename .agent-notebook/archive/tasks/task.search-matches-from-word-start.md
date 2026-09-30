---
id: task.search-matches-from-word-start
type: task
state: closed
title: Search matches words from their start
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
from: task.records-live-with-their-task
link: follows decision.search-matches-words-from-their-start
created: 2026-09-30
updated: 2026-09-30
closed: 2026-09-30
---

Implement decision.search-matches-words-from-their-start in the one match shared by recall and list --match: every word of the phrase must start a word in the id, title, tags, people or body. The recall reply lists every matching id unbudgeted and bodies within the budget.
- 2026-09-30 Maksim Yaromin/claude-code: Implemented in the Core Admission shared by list --match and recall: the text splits on whitespace and a record matches when every word starts a word of its id, title, tags, people or body, case folded; a word that opens with punctuation carries its own boundary. On this notebook, Decisions and Notes with the archive: `lock` 45 -> 14, `concurren` 8. A recall phrase is now a search: every matching id is listed whatever the budget, only bodies give way (the recall reshaping landed together with task.the-session-opens-with-work-and-rules). Docs: replies (narrowing), ideas guide.
- 2026-09-30 Maksim Yaromin/claude-code: list --match and a recall phrase share one match: every word must start a word of the id, title, tags, people or body. A recall phrase is a search that lists every match. On this notebook lock went from 45 to 14 matches. PR #96.
