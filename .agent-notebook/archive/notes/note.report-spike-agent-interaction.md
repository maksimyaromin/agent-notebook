---
id: note.report-spike-agent-interaction
type: note
state: retired
title: Report: Spike: agent interaction
by: Maksim Yaromin
via: claude-code
from: task.spike-agent-interaction
created: 2026-08-29
updated: 2026-08-29
---

# s3 — Spike: agent interaction — report

Closed 2026-08-25, approved after a design review (round 1 settled: all six frontier decisions as recommended, plan accepted whole).

Primary deliverable: the interaction spec, approved.

Settled decisions: shape-matched plain output (plain table for lists, key: value for records, designed composite for Status; `--json` compact everywhere; TOON rejected finally — literature plus own measurement showing +19.4% worse than compact JSON on non-uniform lists); structured errors `error[code]` + computed `try:` lines in one two-section catalog; gated Status injection (full dashboard only on signal, quiet line otherwise, `compact`-matcher re-injection, micro-Status in every mutation reply); minimal injection-safety posture (titles-only, framing line, documented residual, re-open clause before any remote surface); `anb setup` project-scoped by default with explicit `--global`, AGENTS.md snippet floor for all four agents, hooks for Claude Code + Codex in v1, no OpenCode plugin code, Pi extension deferred, marker-bounded idempotent install/remove, fail-soft; skill generated from the CLI's single source of truth with a byte-diff CI gate, worked examples + error catalog, ~600-word length as instrumented hypothesis.
