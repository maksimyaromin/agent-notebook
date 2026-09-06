---
id: note.report-status-json-carries-debt-as
type: note
state: retired
title: Report: status --json carries Debt as fields
by: Maksim Yaromin
from: task.status-json-debt-fields
created: 2026-09-06
updated: 2026-09-06
---


# status --json carries Debt as fields

`anb status --json` rendered each Debt row as the code and the display line, so a program wanting the two ids of a `may-conflict` pair or the days of a clock parsed the sentence the tool had composed a moment earlier. `--json` is the reply contract's promise to programs, and a row that is a sentence keeps the promise in form and breaks it in content.

## What changed

- Each Debt row carries the signal's own fields between `code` and `line`: `id` and `days` for the four clocks; `id` with `origin`, `date`, `target` or `proof` for the record-bound signals; `pair` for `may-conflict` and `project` with `global` for `shadow`, each side a cited record in the `{id, by, via}` shape the `add` reply already uses for its `may-conflict` rows, so one parser serves both; `file` and `errors` for `invalid`, as `check` findings name a file.
- Rendering only: the Debt model, the text output, the row bound per class and `count` are as they were, and `line` stays beside the fields.
- The Status reference gains a JSON fields column in the Debt table and states the row shape in its JSON section.

## Evidence

- A CLI test builds a notebook with a stale Task, a dangling mention, an undeclared Decision pair and an invalid file, and asserts the fields of each row from the promise: 27 days between the last touch and the test's day, the target the body cites, the pair in the order the line prints it with `via` present only where the record carries it, one error finding for an invalid state. The test was shown red once on a deliberately wrong expectation.
- `scripts/check.sh` green; `pnpm docs:check` green.
- Smoke check: no defect. Should-fix, taken: the doc comment on the row renderer restated its body; only the contract clause survives.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/50, squash-merged on a green CI check.
