---
id: task.public-text-carries-no-internal
type: task
state: review
title: Public text carries no internal information: a sweep of the book, the README, the skills and the scripts
by: Maksim Yaromin
from: task.release-gate-v1
tags: docs
priority: 1
created: 2026-09-05
updated: 2026-09-05
---

Owner ruling (2026-09-05): the documentation is written for strangers, not for the owner. It must carry no intimate, personal or internal information: no account names, no token locations or file names where secrets live, no description of the maintainer's machine or shell, no dated rulings, no names of people or models, no process that is the owner's own. Found already: docs/contributing/releasing.md describes the publish token in a git-ignored .env and names the npm account; scripts/release/publish.sh hardcodes the account and explains the maintainer's shell in comments. Sweep every public text: docs/**, README.md, AGENTS.md, packages/*/README.md, .agents/skills/**, scripts/**, the workflows' comments, the help strings. What a contributor needs stays, written for them; what is the maintainer's moves to the working documents under .tmp or to CLAUDE.local.md. The sweep is itself held to the engineer's writing rules: text quality pass, humanizer, plain engineering English.
- 2026-09-05 Maksim Yaromin: First pass done: the releasing page describes the tooling and no longer the maintainer's setup; publish.sh reads the expected account from .env (NPM_PUBLISHER) instead of hardcoding it and its comments describe what the script needs, not the maintainer's shell; the Pages comment speaks of the repository's secrets; 'the owner's taste' and 'the way the owner does' became the project's. The committed notebook itself is the owner's decision: question.does-the-committed-notebook-go-public-as, born from this task. A second sweep by another model is running.
- 2026-09-05 Maksim Yaromin: Second pass clean. Report at .tmp/docs/report-public-text-sweep.md.
