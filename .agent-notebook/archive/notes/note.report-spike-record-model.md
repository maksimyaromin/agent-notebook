---
id: note.report-spike-record-model
type: note
state: retired
title: Report: Spike: record model
by: Maksim Yaromin
via: claude-code
from: task.spike-record-model
created: 2026-08-29
updated: 2026-08-29
---

# s2 — Spike: record model — report

Closed 2026-08-25, approved after a design review (round 1 settled, frontier empty).

Primary deliverable: the record-model spec, approved.

Settled decisions: four types confirmed (persistence-semantics axis); Note kinds fact|term|guide; lifecycles and enums fixed per type with optional review, first-class reopen, structural Question routing (`routed-to`); hold (reason-mandatory) vs computed blocked kept as two mechanisms; six new envelope fields (via, priority, hold, hold-until, review-by, routed-to); origin-keyed aging as config-driven Debt clocks; by+via authorship; mention index as on-demand regex query; undeclared conflicts surfaced mechanically, never auto-resolved.
