---
id: task.npm-distribution
type: task
state: closed
title: npm distribution
by: Maksim Yaromin
via: claude-code
tags: dist
link: note note.report-npm-distribution
blocked-by: task.cli-task-cycle
blocked-by: task.milestone-cli-complete
blocked-by: task.skills
created: 2026-08-29
updated: 2026-09-05
closed: 2026-09-05
---

Package @supolka/agent-notebook shipping platform Rust binaries (esbuild/turbo distribution pattern; swc/biome as Rust precedents); npx -y works from zero.
- 2026-09-05 Maksim Yaromin: Built: the shim package @supolka/agent-notebook (bin/anb.js resolving one of five platform packages, forwarding argv, stdio and the exit code), five platform packages with os/cpu, scripts/release (check-versions, place-binary, fetch-binaries, publish), release.yml (five-runner build, publish job with dry-run gate and OIDC permission), docs/contributing/releasing.md. Verified: shim answers anb 0.1.0 and forwards a refusal's exit 1; publish.sh dry run green, the token read from its file; docs check green. Review running.
- 2026-09-05 Maksim Yaromin: Review: no must-fix; platform packages gained homepage. The maintainer's steps: push v0.1.0, fetch the run's binaries, publish.sh --publish with the OTP, then the Trusted Publisher and RELEASE_DRY_RUN=false.
