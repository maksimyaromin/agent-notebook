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

Closed 2026-08-25, owner-approved after a Lavish grill review (round 1 settled, frontier empty).

Primary deliverable: `.tmp/docs/spec-anb-record-model.md` (owner-approved record-model spec).

Also delivered: ADR 0005 (accepted) `.tmp/docs/adr/0005-record-model.md`; concept-spec updates (record model, origin-keyed aging, mentions/conflicts, authorship bullets; reachability S2 question answered; theme closed); CONTEXT.md (+Hold, +Mention, Debt extended); research sweeps `.tmp/docs/research/09..11-s2-*.md` (47 verified cards); owner's S1 correction applied repo-wide: notebook directory is `.agent-notebook/`, CLI stays `anb`.

Settled decisions: four types confirmed (persistence-semantics axis); Note kinds fact|term|guide; lifecycles and enums fixed per type with optional review, first-class reopen, structural Question routing (`routed-to`); hold (reason-mandatory) vs computed blocked kept as two mechanisms; six new envelope fields (via, priority, hold, hold-until, review-by, routed-to); origin-keyed aging as config-driven Debt clocks; by+via authorship; mention index as on-demand regex query; undeclared conflicts surfaced mechanically, never auto-resolved.
