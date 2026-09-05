---
id: task.npm-distribution
type: task
state: review
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
- 2026-09-05 Maksim Yaromin: Built: the shim package @supolka/agent-notebook (bin/anb.js resolving one of five platform packages, forwarding argv, stdio and the exit code), five platform packages with os/cpu, scripts/release (check-versions, place-binary, fetch-binaries, publish), release.yml (five-runner build, publish job with dry-run gate and OIDC permission), docs/contributing/releasing.md. Verified: shim answers anb 0.1.0 and forwards a refusal's exit 1; publish.sh dry run green as maksimy reading the token from .env only; docs check green. Smoke check running.
- 2026-09-05 Maksim Yaromin: Smoke check: no must-fix; platform packages gained homepage. Report at .tmp/docs/report-npm-distribution.md. The owner's steps: push v0.1.0, fetch the run's binaries, publish.sh --publish with the OTP, then the Trusted Publisher and RELEASE_DRY_RUN=false.
