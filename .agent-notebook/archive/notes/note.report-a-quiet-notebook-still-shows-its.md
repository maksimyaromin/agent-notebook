---
id: note.report-a-quiet-notebook-still-shows-its
type: note
state: retired
title: Report: A quiet notebook still shows its standing rules
by: Maksim Yaromin
from: task.a-quiet-notebook-still-shows-its
created: 2026-09-08
updated: 2026-09-08
---

# Report: a quiet notebook still shows its standing rules

Closes [issue 66](https://github.com/maksimyaromin/agent-notebook/issues/66): a notebook holding rule Decisions and no Task printed the quiet line, so the session hook delivered nothing of the law until the first Task existed.

## Result

A standing rule is a signal on its own. The gate before the budget ladder counts live `rule` Decisions beside active, review, ready and Debt, so a notebook of rules and no Task opens the full dashboard: the counts line, `rules[N]`, the budget line. The hook payload carries the same text. The `rules→count` rung of the ladder is untouched, as the issue asked.

A Decision that is not a rule, and a Note, still leave the notebook quiet: neither binds a session at its start.

Of the two shapes the issue offered, the gate change was taken over "quiet line, then rules": one dashboard shape keeps the reply contract, the budget line still reports what the dashboard cost, and a line saying "quiet" followed by a section would say two things at once.

## Evidence

- `./scripts/check.sh` green; `pnpm docs:check` green.
- Core tests: a rule alone opens the gate and prints its row; a shape Decision and a fact Note alone stay quiet. CLI test: the text dashboard and the hook payload both carry `rules[1]` with no Task in the notebook. The gate test proved red on the inverted expectation.
- Book: `docs/reference/status.md` names the rule among the signals; `docs/guides/session.md` says a rule alone opens the summary.

## Limits

- Quiet stays the word for a notebook with nothing binding at all; a notebook of Notes and non-rule Decisions is quiet, and its knowledge is a `list` away.
