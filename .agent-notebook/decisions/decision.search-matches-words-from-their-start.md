---
id: decision.search-matches-words-from-their-start
type: decision
state: active
kind: shape
title: Search matches words from their start; meaning is the agent's
by: Maksim Yaromin
via: claude-code
created: 2026-09-30
updated: 2026-09-30
---

`recall "<phrase>"` and `list --match` share one match: the phrase splits into words, and a record matches when every word starts a word in its id, title, tags, people or body, case folded. `lock` finds `locks` and `locking` but not `block`. The reply carries every matching id without a budget and bodies only within it, in a deterministic order.

There is no semantic search. An embedding model would add a download or a network dependency, an index that drifts from the Markdown, and results that change with the model, while a notebook holds hundreds of records that the calling agent can judge by meaning itself. The skill tells the agent to turn a meaning into domain words and run several searches.

Measured on this notebook with the 0.9.0 substring match, across Notes and Decisions including the archive: `lock` matched 45 records, almost all through `block` and `blocked-by`, while `concurren` matched the 7 relevant ones.
