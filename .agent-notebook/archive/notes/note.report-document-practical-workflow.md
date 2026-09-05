---
id: note.report-document-practical-workflow
type: note
state: retired
title: Report: Document practical workflow customization
by: Maksim Yaromin
from: task.document-practical-workflow
created: 2026-09-05
updated: 2026-09-05
---

# Practical workflow customization

Added docs/guides/customization.md with recipes for private project memory in .tmp/xxx, environment and command-level notebook selection, relocating a notebook, maintaining custom skill files across setup runs, requiring human review, and changing task organization. The private-memory recipe covers both git exclusion and the skill's default commit instruction, with consistent environment selection for agent commands and hooks.

Linked the guide from README, quickstart, agent setup and personal-notebook documentation, and registered it in the site sidebar. The setup page now points to the recipes instead of repeating partial instructions.

Verified the recipes in a temporary git repository: a Status read creates no notebook; a write uses the configured root; git ignores the private records; subdirectory commands and hooks read the same notebook; the explicit path flag works; and setup preserves customized skills in both installation locations. Documentation build and links pass, with the build warnings already tracked in task.resolve-documentation-build-warnings. git diff --check passes.
