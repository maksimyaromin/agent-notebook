---
id: task.npm-distribution
type: task
state: open
title: npm distribution
by: Maksim Yaromin
via: claude-code
tags: dist
blocked-by: task.cli-task-cycle
blocked-by: task.milestone-cli-complete
blocked-by: task.skills
created: 2026-08-29
updated: 2026-09-05
---

Package @supolka/agent-notebook shipping platform Rust binaries (esbuild/turbo distribution pattern; swc/biome as Rust precedents); npx -y works from zero.
- 2026-09-05 Maksim Yaromin: Owner rulings 2026-09-05. Publishing: the granular token agent-notebook-publish (account maksimy, scope @supolka, 90-day cap) lives only in this repository's git-ignored .env; the owner's shell carries an unrelated NPM_TOKEN for another scope, so nothing here may read a variable of that name or write an .npmrc that references it. The first publish is the owner's, locally, with the OTP prompt, because Trusted Publishing and npm trust both require the package to exist. After it the owner adds a Trusted Publisher (maksimyaromin / agent-notebook / release.yml) and CI publishes by OIDC with id-token: write and no secret. The marathon delivers: the package with bin anb, a publish script that loads .env, passes the token explicitly and refuses unless npm whoami answers maksimy, a green npm publish --dry-run, and the release workflow that builds platform binaries on a tag; the owner pushes the tag. Provenance is off while the repository is private.
