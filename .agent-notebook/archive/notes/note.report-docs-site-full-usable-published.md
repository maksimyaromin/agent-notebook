---
id: note.report-docs-site-full-usable-published
type: note
state: retired
title: Report: Docs site: full, usable, published on Cloudflare Pages
by: Maksim Yaromin
from: task.docs-site-full-usable-published-on
created: 2026-09-05
updated: 2026-09-05
---

# The documentation site (2026-09-05)

Report for task.docs-site-full-usable-published-on. A static site built from the repository's `docs/` tree, deployed to Cloudflare Pages by a GitHub Actions workflow the way a sibling project does it; publishing itself is the maintainer's step.

## What shipped

```
docs/                        the book, GitHub-renderable markdown, 16 pages
  index.md                   the splash page
  start/                     what it is, quickstart
  guides/                    the session, tasks and the shape of work, decisions/notes/questions, wiring agents, drawing the notebook, your own notebook
  reference/                 commands*, records and files, status and debt, replies, refusals and findings*
  contributing/development.md
  llms.md
apps/docs/                   Astro 7 + Starlight 0.42 app reading ../../docs in place (astro-mermaid, starlight-llms-txt)
tools/docs/check.mjs         pnpm docs:check
scripts/docs-reference.sh    renders the two starred pages from `anb skill`; --check in the gate
.github/workflows/ci.yml     a docs job: install, docs:check
.github/workflows/pages.yml  build and `wrangler pages deploy` on push to main
package.json, pnpm-workspace.yaml, pnpm-lock.yaml, .nvmrc (24)
```

Two pages are written by the binary. `scripts/docs-reference.sh` runs `anb skill` into a temporary directory and turns the commands and the refusals references into docs pages (Starlight frontmatter in place of the skill's, the title and contents list dropped); with `--check` it diffs the committed pages against the rendering, and the gate runs that check, so the site cannot list a flag the tool does not have. Every other command block in the guides is the literal output of the binary, captured from scratch runs today.

`pnpm docs:check` holds three rules: the sidebar reaches every page under `docs/`, every relative link in a page resolves to a file and a named heading, and after the build every `href` and `src` a built page makes to the site names a file the build wrote. The CI job runs it on every pull request; the Pages workflow deploys only from main and only when the docs or the app changed.

The app mirrors the sibling's: the same collection loader over `../../docs`, the same relative-link rewriter so a link written as a path on disk (which GitHub renders) becomes the route the site serves, the same edit-link middleware, and the llms.txt plugin publishing `/llms.txt`, `/llms-small.txt` and `/llms-full.txt`. Versions were checked against the registry today: Astro 7.3.1, Starlight 0.42.0, astro-mermaid 2.1.0, starlight-llms-txt 0.11.0; Node 24 is the current LTS and what the machine runs.

## What the maintainer does

1. Create the Cloudflare Pages project `agent-notebook` (direct upload) and set `CLOUDFLARE_API_TOKEN` (Pages edit) and `CLOUDFLARE_ACCOUNT_ID` as repository secrets. The workflow then deploys on the next push to main that touches the docs, or on `workflow_dispatch`.
2. Point `agent-notebook.supolka.dev` at the Pages project, or change `apps/docs/src/site.js` and the README link to the address chosen.

## Verified

The site builds from a clean install and `pnpm docs:check` passes: 16 pages reachable, every link resolves, the built site is whole. The full Rust gate passes with the new reference check. The built site was served locally and opened in Chrome: the splash page, the guides and the reference render, the lifecycle diagrams draw, search indexes 17 pages.

## Review

One review pass, running every command block of every page in a fresh project and holding every prose claim to the code and the binary. Every block in the front page, the quickstart, the session, tasks and your-own-notebook guides and all fifteen refusal examples reproduced byte for byte; the envelope table, the config defaults, the Status section order, the graph fields, the exit codes and the setup table all held. Eight must-fix in prose, all taken: the may-conflict example did not reproduce in page order and the rule was half stated (two shared tags or a citation at write time; a citation alone in Status); the collapse order was told backwards (the queue's rows go first, the log and the held and review rows last); the Debt table was not in print order; two setup outcomes were missing (`links AGENTS.md`, `imports AGENTS.md`); a Note has no `superseded` state; the llms.txt page overstated what the plugin writes; the hook header was misquoted and a missing notebook gets the quiet line rather than an empty payload. One should-fix: a refusal's JSON carries `findings` only when a record's findings caused it. One nit: the gate script's opening comment was stale.
