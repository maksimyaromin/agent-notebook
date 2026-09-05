---
id: note.marathon-of-2026-09-05-every-pull
type: note
state: active
kind: fact
title: Marathon of 2026-09-05: every pull request, hold and Question
by: Maksim Yaromin
from: task.release-gate-v1
tags: marathon
created: 2026-09-05
updated: 2026-09-05
---

# The marathon of 2026-09-05

One session, from the vocabulary decision to a release-ready repository. What remains is the owner's own hands: the first npm publish, the Cloudflare secrets and domain, the history push, the tag, the repository's visibility.

## Pull requests, in order

| PR | What it shipped |
|---|---|
| #7 | the CLI speaks one plain word per concept; a Task ends without work by `--reason`; a Question closes `--resolved-by` |
| #8 | the write paths and check read the user's notebook too |
| #9 | a held Task waits in its own Status section, never as the active one |
| #10 | `anb setup` wires the agents' session start: the snippet, the hooks, verified live in Claude Code and Codex |
| #11 | the skill is rendered by the binary, installed by setup, and diffed in CI |
| #12 | the skill's references live under `references/`; SKILL.md in the standard sections; skill prose passes the humanizer |
| #13 | the atlas skill: the notebook drawn from `anb graph --json`, the intent loop; verified by a fresh session drawing this notebook |
| #14 | help strings and refusal messages read as plain sentences |
| #15 | the README and the MIT license |
| #16 | the documentation site: `docs/` read in place by Astro Starlight, the reference pages rendered from the binary, the docs check, the Pages workflow |
| #17 | npm distribution: the shim package, one package per platform, the release scripts, the Release workflow |
| #18 | fix: the npm packages stay out of the pnpm workspace |
| #19 | fix: the Pages workflow builds and stops until the Cloudflare secrets exist |
| #20 | AGENTS.md for a public repository; the owner's process in `CLAUDE.local.md` |
| #21 | public text carries no maintainer's setup |
| #22 | the skills name mistakes the tool makes easy; the atlas page stays out of the map's way |
| #23 | counts read as English: one task, two tasks |

Two runs went red and were fixed forward the same hour: the docs job on #17 (the frozen lockfile) and the Pages deploy without secrets; since then every merge checks the CI result by exit code before merging.

## Decisions recorded

decision.the-cli-speaks-one-plain-word-per (the four criteria for every word), decision.the-cli-keeps-a-record-s-invariants-not (the CLI keeps a record's invariants, not a team's workflow), decision.the-cli-binary-is-named-anb. ADR 0006 carries the five answers to why Rust, for the owner to accept at the final review.

## Held, with reasons

- task.import-surface-adopt-an-existing-tracker: adoption horizon, no external adopter yet.
- task.main-ruleset-when-plan-allows: waits for the repository to go public; the ruleset is a convention until then.

## Questions for the owner

- question.does-the-committed-notebook-go-public-as: the notebook under `.agent-notebook/` is committed and carries the owner's name, dated rulings and the models' names in report Notes. Public as it is, stripped, or kept out of the public tree.

## The owner's hands

1. Cloudflare: `gh secret set CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` on the repository; `wrangler pages project create agent-notebook --production-branch main` once; `gh workflow run pages.yml`; the custom domain `agent-notebook.supolka.dev` in the dashboard.
2. History: `.tmp/release/README.md`; the scrub is proven on clones of origin with both backends; the force-push with the lease is the owner's, before the first tag.
3. npm: push `v0.1.0` after the scrub; `scripts/release/fetch-binaries.sh <run-id>`; `scripts/release/publish.sh` then `--publish` (the `.env` carries `NPM_TOKEN` and `NPM_PUBLISHER`); then the Trusted Publisher and `RELEASE_DRY_RUN=false`.
4. Accept or reopen the why-Rust answers in ADR 0006 and the MIT license; answer the notebook Question; flip the repository public; the `main` ruleset then.

## Left in the notebook

The release hub task.release-gate-v1 stays open and becomes ready when its last child closes; closing it is the release. task.anb-v1 waits on the release hub and the held import surface.
