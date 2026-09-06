---
id: note.report-present-the-product-in-the-readme
type: note
state: retired
title: Report: Present the product in the README with the authors explanatory style
by: Maksim Yaromin
from: task.readme-product-voice
created: 2026-09-06
updated: 2026-09-06
---

# README product voice

The README presents agent-notebook as a product while retaining the author's explanatory approach: purpose first, then the mechanisms and concrete choices available to users. Its opening describes continuity across sessions and agents, the deterministic CLI, the supplied working method and the ability to change that method. Personal motivation is no longer the README's framing. A short author credit links to the maintainer's public profile.

The documentation landing page uses the same product framing. First-person explanations of design choices remain in the explanatory documentation. Default storage and git behavior, practical customization, CLI guarantees and documentation site links are preserved.

Validation: pnpm docs:check passed without warnings; README documentation routes resolve in the site build; git diff --check passed. No command or runtime behavior changed. No commit or publication was performed.
