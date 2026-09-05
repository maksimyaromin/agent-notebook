---
id: task.public-text-carries-no-internal
type: task
state: open
title: Public text carries no internal information: a sweep of the book, the README, the skills and the scripts
by: Maksim Yaromin
from: task.release-gate-v1
tags: docs
priority: 1
created: 2026-09-05
updated: 2026-09-05
---

Owner ruling (2026-09-05): the documentation is written for strangers, not for the owner. It must carry no intimate, personal or internal information: no account names, no token locations or file names where secrets live, no description of the maintainer's machine or shell, no dated rulings, no names of people or models, no process that is the owner's own. Found already: docs/contributing/releasing.md describes the publish token in a git-ignored .env and names the npm account; scripts/release/publish.sh hardcodes the account and explains the maintainer's shell in comments. Sweep every public text: docs/**, README.md, AGENTS.md, packages/*/README.md, .agents/skills/**, scripts/**, the workflows' comments, the help strings. What a contributor needs stays, written for them; what is the maintainer's moves to the working documents under .tmp or to CLAUDE.local.md. The sweep is itself held to the engineer's writing rules: text quality pass, humanizer, plain engineering English.
