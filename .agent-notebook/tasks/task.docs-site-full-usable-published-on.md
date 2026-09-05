---
id: task.docs-site-full-usable-published-on
type: task
state: open
title: Docs site: full, usable, published on Cloudflare Pages
by: Maksim Yaromin
via: claude-code
from: task.release-gate-v1
tags: docs
blocked-by: task.skills
priority: 2
created: 2026-09-05
updated: 2026-09-05
---

The Definition of Done says docs published. A static site built from the repository's docs, deployed by a GitHub Actions workflow to Cloudflare Pages the way the codemode-executor repository already does it, covering: the record model, the file format a human reads, every command with its output and errors, the skill and setup for each agent, the global notebook. Acceptance: the site builds from a clean checkout in CI; every page is reachable from the front page; nothing on it contradicts the binary at HEAD. Publishing itself needs the owner's Cloudflare account and is the owner's step; the marathon leaves the workflow and the site ready.
