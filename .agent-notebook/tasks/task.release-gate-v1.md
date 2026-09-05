---
id: task.release-gate-v1
type: task
state: open
title: Release gate v1
by: Maksim Yaromin
via: claude-code
tags: gate
blocked-by: task.milestone-self-host-switch
blocked-by: task.skill-from-help-ci-drift-check
blocked-by: task.npm-distribution
blocked-by: task.cli-decide-note-ask-answer
blocked-by: task.cli-check-archive-edit-search-overview
blocked-by: task.gate-final-cli-name
blocked-by: task.why-rust-dossier-release-gate
blocked-by: task.git-reconciliation
blocked-by: task.github-dev-flow-actions-ci-fmt-clippy-te
blocked-by: task.global-notebook
blocked-by: task.skills
blocked-by: task.readme-the-page-a-stranger-reads-first
blocked-by: task.docs-site-full-usable-published-on
blocked-by: task.agents-md-rewritten-for-a-public
blocked-by: task.skills-common-mistakes-that-are-real-and
created: 2026-08-29
updated: 2026-09-05
---

Full Definition of Done: anb self-hosts its own development; SessionStart hooks deliver Status within Budget in Claude Code and Codex; negative corpus green; npx zero-install works; README/docs published; skill CI-checked; why-Rust answers recorded.
- 2026-09-05 Maksim Yaromin: Marathon of 2026-09-05 done: every child closed, main green on CI, docs and Pages. The hub's acceptance close is the release itself and stays with the maintainer: the Cloudflare secrets and the domain, the tag v0.1.0, the first npm publish with scripts/release/publish.sh, the Trusted Publisher, the repository's visibility, the ruleset.
- 2026-09-05 Maksim Yaromin: Documentation site live: Cloudflare Pages project agent-notebook deploys from pages.yml on every push to main that touches the book; custom domain agent-notebook.supolka.dev attached with a proxied CNAME and serving.
- 2026-09-05 Maksim Yaromin: Published: GitHub release anb v2026.09.05 with one archive per platform and SHA256SUMS; @supolka/agent-notebook 0.1.0 and its five platform packages on npm, public; npx -y @supolka/agent-notebook answers anb 0.1.0 from an empty directory. Left: the Trusted Publisher on npm for the six packages, RELEASE_DRY_RUN=false, the repository's visibility, the main ruleset.
