---
id: decision.an-epic-is-the-pairing-a-tag-is-for
type: decision
state: active
kind: rule
title: An epic is the pairing; a tag is for finding one
by: Maksim Yaromin
via: claude-code
from: question.how-is-an-epic-assembled-only-from-the
tags: cli
created: 2026-08-30
updated: 2026-08-30
---

Hub detection stays the two-sided pairing: a Task blocked by a record that also carries it as Origin. A Task blocked by work it did not give birth to is a gate, not an epic, and belongs on the dashboard as the open Task it is — widening detection to any blocked Task would make every blocked Task an epic and spend the opening on the notebook's whole dependency graph. The tension with tags dissolves rather than being tie-broken: a hub wearing tags epic, gate or milestone is how a session finds it by name through search, while the edges are what the dashboard reads. Detection never reads a tag, and decomposition is never written as one. A hub that loses its last born-inside child stops being an epic, which is the same rule read backwards and not a separate defect.
