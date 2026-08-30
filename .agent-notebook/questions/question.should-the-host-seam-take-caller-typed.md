---
id: question.should-the-host-seam-take-caller-typed
type: question
state: open
title: Should the host seam take caller-typed paths as &Path instead of &str?
by: Maksim Yaromin
from: task.graph-emit-html
created: 2026-08-30
updated: 2026-08-30
---

The host seam takes caller-typed paths as `&str` (`read_report`, `write_artifact`), so any path that is not UTF-8 has to be refused rather than served; the two options are widening that seam to `&Path` end to end, or keeping the refusal as the tool's answer.
