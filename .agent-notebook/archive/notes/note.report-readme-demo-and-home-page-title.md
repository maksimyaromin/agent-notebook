---
id: note.report-readme-demo-and-home-page-title
type: note
state: retired
title: Report: The README opens with a demo and the home page carries its own title
by: Maksim Yaromin
from: task.readme-demo-and-home-page-title
created: 2026-09-06
updated: 2026-09-06
---

# The README opens with a demo and the home page carries its own title

The README showed the tool only in prose and one Status excerpt; a reader decides on a README in seconds, and a terminal recording shows the whole cycle in one look. The home page of the book rendered its browser title as `agent-notebook | agent-notebook`, because Starlight joins the page title and the site title without exception and the page was named after the site.

## What shipped

- `apps/docs/public/demo.gif`, a 32 second recording of the quickstart cycle on the built 0.3.0 binary: a Task is added, started and logged, a rule is recorded, the next session reads Status, the Task closes with its report and is archived, and the check answers `count: 0`. The README shows it under the badges.
- `docs/index.md` titles the page `Working memory for coding agents` and keeps `agent-notebook` as the hero title, so the browser title reads `Working memory for coding agents | agent-notebook` and the H1 is unchanged.
- The recording is made by hand from a vhs tape kept outside the repository, with the owner's terminal font; the tape and the font are not published.

## Evidence

- `pnpm docs:check` green; the built `apps/docs/dist/index.html` carries `<title>Working memory for coding agents | agent-notebook</title>`, `og:title` with the tagline and the H1 `agent-notebook`.
- `./scripts/check.sh` green, `anb check` at `count: 0`.
- Every frame of the recording is real output of the binary; the three frames inspected (the four commands, Status, the closing) show no wrapped line and no empty screen.
- Smoke check by a second model: see the findings below.

## Smoke check findings

- The built title and H1 verified as claimed; the README image path resolves.
- The alt text omitted the archive step the recording shows; it now names it.
- Three claims concerned the public tape and the sentence in the development page; both were withdrawn by the owner's ruling before the check ended, so those findings do not apply to the change as shipped.
