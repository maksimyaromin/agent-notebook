---
id: question.should-the-host-seam-take-caller-typed
type: question
state: closed
title: Should the host seam take caller-typed paths as &Path instead of &str?
by: Maksim Yaromin
from: task.graph-emit-html
reason: write_artifact went with the emitted HTML; read_report is the one seam left, fed by --note and --report, and a non-UTF-8 path is refused by the argument parser before the seam is reached. macOS enforces UTF-8 file names and a typed path is UTF-8 anyway; the refusal names the argument and is the tool's answer.
created: 2026-08-30
updated: 2026-09-02
---

The host seam takes caller-typed paths as `&str` (`read_report`, `write_artifact`), so any path that is not UTF-8 has to be refused rather than served; the two options are widening that seam to `&Path` end to end, or keeping the refusal as the tool's answer.
