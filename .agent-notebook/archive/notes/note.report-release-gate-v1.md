---
id: note.report-release-gate-v1
type: note
state: retired
title: Report: Release gate v1
by: Maksim Yaromin
from: task.release-gate-v1
created: 2026-09-06
updated: 2026-09-06
---

# Release gate v1

The gate is passed. Every item of its Definition of Done is in place and can be checked.

- anb hosts its own development: the backlog is `.agent-notebook/` in this repository, changed only through the tool. Fifty-seven closed Tasks sit in the archive, each with its report as a Note.
- SessionStart hooks deliver Status within the budget: `anb setup` writes the hook for Claude Code and Codex, `anb status --hook` runs when a session opens, and Status stays under its default of 1500 tokens.
- The gate CI runs is green on main: format, clippy with warnings as errors, tests, doctests, rustdoc, the rendering diffs and the docs check.
- npx works with nothing installed: `npx -y @supolka/agent-notebook --version` answers `anb 0.1.1` from an empty directory.
- The README and the book are published: the README at the repository root, the book at agent-notebook.supolka.dev, deployed by Cloudflare Pages from `pages.yml` on every push to main that touches it.
- The skill is CI-checked: `anb skill` renders the `anb` skill from the binary, and CI fails when the committed copy under `.agents/skills/anb` drifts from the rendering.
- The why-Rust answers are recorded: the report of task.why-rust-dossier-release-gate is a Note in the archive.

The release itself: two releases on GitHub, `anb v2026.09.05` and `anb v2026.09.06`, each with one archive per platform, a `SHA256SUMS` file and the changelog entry as its notes. On npm, `@supolka/agent-notebook` 0.1.0 was published from a maintainer's machine and 0.1.1 by the Release workflow through npm trusted publishing, with no token anywhere; a Trusted Publisher is set on each of the six packages and `RELEASE_DRY_RUN` is false. The repository went public on 2026-09-06 with the ruleset on main in force, and the project's mark stands at the top of the README and beside the site title.
