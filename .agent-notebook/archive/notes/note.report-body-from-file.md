---
id: note.report-body-from-file
type: note
state: retired
title: Report: add and edit take the body from a file or stdin
by: Maksim Yaromin
from: task.body-from-file
created: 2026-09-06
updated: 2026-09-06
---

# add and edit take the body from a file or stdin

`add --body` and `edit --body` took the body as one shell argument, so a model of nine paragraphs passed through every quoting rule of the shell, a body that began with a flag-shaped word was parsed as a flag until it was spelled `--body=…`, and changing one word of a record meant reading the body back and passing all of it to `edit --body`. `close --note <file>` already read a report from a file, so the asymmetry was visible on the same day's work.

## What changed

- `--body-file <path>` beside `--body` on `add` and `edit`; the two are refused together by the command line; `-` reads standard input. The text lands where `--body` lands, so no new invariant: the Core sees a body and nothing else.
- The shell reads the file through the one seam it already reads a report through, so `close --note -` reads standard input by the same rule. A path that cannot be read is refused under the flag that named it, as `--note` always was.
- The skill's domain section teaches the flag for a model or a spec, the knowledge guide names it beside `edit --body`, the domain guide shows a model added from a file. The commands reference and the skill were regenerated.

## Evidence

- CLI tests: a model added from a file lands as its body byte for byte; an edit takes the whole body from `-`; a file that is not there is a recovery payload with the try line and writes nothing; `--body` beside `--body-file` is a command-line conflict. A process test pipes a spec into `add --body-file -` through the real binary. Each was shown red once.
- `scripts/check.sh` green, the regenerated skill and reference pages included; `pnpm docs:check` green.
- Smoke check by another model, once: no functional defect. Should-fix, taken: the shared test harness still called the file seam a report reader after the rename, swept to the file it reads. Should-fix, taken: an empty body file on edit clears the body as an empty body does, and a test now pins it. Nit, noted: the flag help names a sibling flag without backticks, as the close help does.

Pull request: https://github.com/maksimyaromin/agent-notebook/pull/52, squash-merged on a green CI check.
