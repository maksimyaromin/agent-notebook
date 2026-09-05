---
id: note.report-write-the-readme-and-book-for
type: note
state: retired
title: Report: Write the README and book for engineers
by: Maksim Yaromin
from: task.write-the-readme-and-book-for-engineers
created: 2026-09-05
updated: 2026-09-05
---

# README and documentation editorial revision

Rewrote the README and authored documentation around the product's foundation: a deterministic CLI with explicit record rules, informative replies, and replaceable workflow skills. Git sharing, notebook location, global knowledge and visualization are presented as choices. Updated the website metadata and generated machine-readable introduction to describe the same product.

The guides cover setup, session handoffs, Tasks and epics, knowledge records, personal knowledge, graph review and workflow customization. The reference corrections describe the Status estimator and minimum output, independent row limits, structural hub detection, manual hold release, creation versus transition retries, JSON result shapes, and recovery after storage failure. The installation instructions now distinguish a global npm install from an npx invocation. Corrected the Status example to match the CLI.

Generated references were updated in the skill generator and regenerated through the binary. No record behavior changed. The existing README edit was included in the editorial source; no commit or publication was made.

## Verification

- `./scripts/check.sh` passed: formatting, clippy, tests, doctests, rustdoc, skill and documentation reference drift checks.
- `pnpm docs:check` passed after the final edits: all 17 documentation pages are reachable and all checked links resolve.
- The quickstart session ran in an isolated temporary notebook; all 11 documented command replies matched, normalizing only the illustrative date.
- `node --check apps/docs/astro.config.mjs` and `git diff --check` passed.
- The documentation build reports oversized-chunk and Starlight i18n/404 warnings. Their investigation is recorded in task.resolve-documentation-build-warnings.

A sandboxed pnpm invocation created a local package store and stalled before the build. That invocation was stopped, its store was moved out of the repository, and the documentation check completed using the normal pnpm environment.
