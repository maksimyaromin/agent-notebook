---
id: question.should-state-vs-residence-disagreement-b
type: question
state: routed
title: Should state vs residence disagreement be a Check finding?
by: Maksim Yaromin
via: claude-code
from: task.cli-check-archive-edit-search-overview
tags: cli
routed-to: task.check-finding-state-vs-residence-mismatc
created: 2026-08-29
updated: 2026-08-29
---

Check reports nothing for an open task sitting in the archive directory, or a closed task sitting live, yet the format contract holds that a record's state field and its live/archive location may disagree only as a named finding. The finding-code catalog is a closed set, so naming this split means widening the catalog — blast radius beyond the archive verb's task, which is what made the split first-class. Found during that task's independent code review.
