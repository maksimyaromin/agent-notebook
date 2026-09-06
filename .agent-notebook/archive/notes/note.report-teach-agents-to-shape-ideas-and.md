---
id: note.report-teach-agents-to-shape-ideas-and
type: note
state: retired
title: Report: Teach agents to shape ideas and maintain a development domain model
by: Maksim Yaromin
from: task.author-method
created: 2026-09-06
updated: 2026-09-06
---

# Teach agents to shape ideas and maintain a development domain model

Added idea, model and spec Note kinds while preserving existing kinds and lifecycle semantics. Reworked the shipped method around source ideas, evidence, proportional planning, domain models, explicit provenance and resumption. Authored guidance remains in crates/anb/src/skill/anb.rs; the generator emits planning and domain references alongside the existing references. Updated public guides, CLI help, reference renderings and setup expectations. Quoted generated YAML descriptions after parsing exposed invalid unquoted colons.

Validation: scripts/check.sh passed, including format, clippy, tests, doctests, rustdoc and generated renderings. pnpm docs:check passed with all 20 book pages reachable and links resolving. All six generated Markdown frontmatters parsed with Ruby YAML. The skill-creator Python validator could not run because PyYAML is unavailable; this is not reported as a pass. The planning example executed in a scratch notebook and selected its prerequisite Task correctly; check returned zero findings.

Two independent scratch exercises covered uncertain feature shaping and a small agreed label change. The first retained uncertainties as Questions without inventing acceptance or implementation work. The second created an idea and one queued Task without unnecessary decomposition. These exercises provide examples of agent behavior, not a guarantee of deterministic interpretation.

The new Note kinds require this updated binary; older releases reject them. No release, commit or push was performed.
