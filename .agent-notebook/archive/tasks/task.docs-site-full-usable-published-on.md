---
id: task.docs-site-full-usable-published-on
type: task
state: closed
title: Docs site: full, usable, published on Cloudflare Pages
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: docs
link: note note.report-docs-site-full-usable-published
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

The Definition of Done says docs published. A static site built from the repository's docs, deployed by a GitHub Actions workflow to Cloudflare Pages the way the codemode-executor repository already does it, covering: the record model, the file format a human reads, every command with its output and errors, the skill and setup for each agent, the global notebook. Acceptance: the site builds from a clean checkout in CI; every page is reachable from the front page; nothing on it contradicts the binary at HEAD. Publishing itself needs the owner's Cloudflare account and is the owner's step; the marathon leaves the workflow and the site ready.
- 2026-09-05 Maksim Yaromin: Site built: docs/ book of 16 pages (start, guides, reference, contributing, llms) read in place by an Astro Starlight app in apps/docs, mirroring codemode-executor; the commands and refusals pages are rendered from anb skill by scripts/docs-reference.sh and diffed in the gate; tools/docs/check.mjs holds the sidebar, every relative link and every reference of the built site; CI gained a docs job and pages.yml deploys to Cloudflare Pages on push to main with the owner's two secrets. Builds clean locally; visual check done; smoke check next.
- 2026-09-05 Maksim Yaromin: Smoke check: eight prose errors against the binary (may-conflict rule and example, collapse order, Debt order, two setup outcomes, Note states, llms.txt contents, hook header and empty-payload claim), one should-fix (refusal JSON shape), one nit; all taken, every command block reproduced. Report at .tmp/docs/report-docs-site.md.
