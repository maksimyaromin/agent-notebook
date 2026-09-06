---
id: task.envelope-is-valid-yaml
type: task
state: open
title: The envelope is valid YAML
by: Maksim Yaromin
via: claude-code
link: issue https://github.com/maksimyaromin/agent-notebook/issues/60
priority: 2
created: 2026-09-06
updated: 2026-09-06
---

Every envelope the tool writes parses as YAML frontmatter: a value a plain scalar cannot carry is double-quoted with YAML escapes, a repeatable key with two or more values is written as a block sequence, and the reader accepts the old forms alongside the new, so every record written before the change still reads. Counted on 2026-09-06: 94 of 177 records carry a value with a colon and a space, 92 of them report titles, and 29 repeat a key. Every write renders the canonical form, so files converge as they are touched; check names the old form as a warning. Decide before writing code whether repeatable keys become sequences or quoting alone is the change; the grammar reference states the promise either way.
