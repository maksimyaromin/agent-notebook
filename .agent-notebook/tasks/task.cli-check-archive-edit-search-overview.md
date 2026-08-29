---
id: task.cli-check-archive-edit-search-overview
type: task
state: open
title: CLI: check / archive / edit / search / overview
by: Maksim Yaromin
via: claude-code
tags: cli
blocked-by: task.core-grammar-parser-renderer
blocked-by: task.core-record-model-invariants
created: 2026-08-29
updated: 2026-08-29
---

Check names file, line, reason; archive moves records with history; search and overview ship in v1. Adapter duty: a non-UTF-8 file must surface through the CLI as the not-utf8 finding — visible to the calling LLM, never a crash or a silent skip; the Core cannot detect it (Storage feeds it strings), so the fs adapter owns it.
- 2026-08-29 claude-code: Field evidence from the self-host migration: imported titles and bodies could not be touched after add — no edit surface — and four dangling-mention debt rows now sit in closed imported bodies with no repair path; edit/archive is what unlocks it.
- 2026-08-29 Maksim Yaromin: Field finding while testing 2026-08-29: 'anb archive' answers with raw clap usage (exit 2), not the structured error contract — no error[<stable-code>], no try: lines. An agent typing an unknown or not-yet-built verb gets no recovery path. When this task ships the archive verb, also decide whether unknown-subcommand errors join the structured catalog (e.g. error[unknown-command] + try: listing the nearest verbs) or stay clap's.
