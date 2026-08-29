---
id: task.slug-minting-cut-at-a-word-boundary
type: task
state: open
title: Slug minting: cut at a word boundary
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
tags: cli
created: 2026-08-29
updated: 2026-08-29
---

Ids minted from long titles truncate mid-word: task.github-dev-flow-actions-ci-fmt-clippy-te and task.epic-pattern-scoped-queries-status-hub-g both came out of the self-host migration with a severed last word. The length cap should land on a hyphen so every kept word survives whole; an explicit --id stays the escape hatch.
