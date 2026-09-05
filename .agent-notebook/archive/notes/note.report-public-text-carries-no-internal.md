---
id: note.report-public-text-carries-no-internal
type: note
state: retired
title: Report: Public text carries no internal information: a sweep of the book, the README, the skills and the scripts
by: Maksim Yaromin
from: task.public-text-carries-no-internal
created: 2026-09-05
updated: 2026-09-05
---

# Public text carries no internal information (2026-09-05)

Report for task.public-text-carries-no-internal, filed when the owner read a docs page that described where their publish token lives and named their npm account. The rule it enforces: the book, the README, the skills, the scripts and the workflows are written for strangers and carry nothing that is the owner's own.

## What changed

An inventory of every committed text outside the notebook, by grep for the tell-tale words (owner, account names, `.env`, shell, dates, model names, rulings) and by reading the pages the first grep pointed at, found the internal information in five places, all now rewritten:

- `docs/contributing/releasing.md` described the maintainer's first publish down to where the token lives, which file holds it and which account owns the scope. It now says that a first publish is made from a maintainer's machine with `scripts/release/publish.sh`, that the script's header says what it needs and how it authenticates, and that it refuses to publish as anyone but the account it expects.
- `scripts/release/publish.sh` hardcoded the account and explained, in its header, the maintainer's shell and the other token it carries. The header now states the script's interface: two lines in a git-ignored `.env`, `NPM_TOKEN` and `NPM_PUBLISHER`, read from the file and never from the environment; the expected account comes from the second line. The refusal message names the fact, not the scope's owner.
- `.github/workflows/pages.yml` called the two Cloudflare secrets "the owner's"; they are the repository's.
- `docs/guides/knowledge.md` and `docs/guides/session.md` said the standing rules apply "the owner's taste" and that terms teach an agent to name things "the way the owner does"; the project's rules and the project's language.

The owner's specifics moved where they belong: the account, the token's name and cap and the shell's other token are in `.tmp/release/README.md`, and `NPM_PUBLISHER=maksimy` joined the repository's git-ignored `.env` so the script keeps working.

What stays, by nature public: the repository's GitHub address in install commands and badges, the copyright line of the license, the `author` of the npm package, and dates inside literal tool replies in examples.

## The notebook

The sweep covers the texts written for readers. The notebook under `.agent-notebook/` is committed by choice and will be public too, and it is the owner's history: every record carries the owner's name in its `by` line, the task bodies carry dated rulings and the course of the marathon, and the report Notes name the models that wrote and checked the code. Whether it goes public as it is, stripped, or kept out of the public tree is the owner's decision, filed as question.does-the-committed-notebook-go-public-as.

## Smoke check

A second pass by another model over every committed text outside the notebook, with the first pass's diff in hand: no occurrence left of an owner, a ruling, a process word, a model name as a person, a secret's location tied to a person or a pointer outside the repository. What remains is public by nature: the repository's address in install commands and badges, the license's copyright line, the package's `author`, dates inside literal replies. The changed texts hold to the writing rules and describe the scripts and workflows truthfully. One nit, a curly apostrophe in a frontmatter description, is the site's convention inside single-quoted YAML and stays.
