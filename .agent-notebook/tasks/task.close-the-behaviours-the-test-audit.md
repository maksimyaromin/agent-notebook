---
id: task.close-the-behaviours-the-test-audit
type: task
state: open
title: Close the behaviours the test audit proved unguarded
by: Maksim Yaromin
tags: testing
priority: 3
created: 2026-08-30
updated: 2026-08-31
---

Six mutations the suite does not catch, each confirmed green under the mutation: (1) a scope computed over the scoped subset with a scope-local resolver — the doc says a scope narrows what is shown, never what is read, and nothing observes it; (2) an unreadable archive destination read as free instead of occupied, so archive would overwrite unreadable history; (3) note_links dedup — a duplicated link: note line makes an archive cascade fail mid-way with a spurious not-found; (4) view's archived: line in text, so the text rendering cannot tell history from live work; (5) the add --id retry offered for a non-Task dangling reference, a try: line that cannot run; (6) the property generator never produces a task already carrying blocked-by, so unblock's erasing replay is never generated. Acceptance: each closed by a test that fails under the named mutation.
- 2026-08-31 Maksim Yaromin: CI review finding (2026-08-31): both crates carry zero doctests, so the gate's doctest step (cargo test --doc) and CI pass vacuously over 0 examples — the 'doctests' criterion is currently proof of nothing.
