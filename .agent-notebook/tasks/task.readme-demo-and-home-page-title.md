---
id: task.readme-demo-and-home-page-title
type: task
state: review
title: The README opens with a demo and the home page carries its own title
by: Maksim Yaromin
tags: docs
priority: 1
created: 2026-09-06
updated: 2026-09-06
---

The README shows the tool only in prose and one Status excerpt; a recorded terminal session shows the whole cycle in one look. The home page of the book renders its browser title as the site name twice, because Starlight joins the page title and the site title and the page is named after the site. The demo is recorded from a tape checked in beside the other scripts, so it is regenerated when a reply it shows changes.
- 2026-09-06 Maksim Yaromin: the title is joined in Starlight's head builder without exception; the page title becomes the tagline and hero.title keeps the name as the H1; the demo tape lives in scripts/ and renders to apps/docs/public/demo.gif
- 2026-09-06 Maksim Yaromin: recorded with vhs 0.11 and its default font into apps/docs/public/demo.gif from scripts/demo.tape; README carries it under the badges; docs/index.md titles the page by its tagline and keeps the name in hero.title, the built head reads 'Working memory for coding agents | agent-notebook'; development.md says how to record again; pnpm docs:check green, anb check 0
- 2026-09-06 Maksim Yaromin: ruling: the tape stays private under .tmp/demo with the owner's font; only the recording is committed and development.md says nothing about it, so the demo is refreshed by hand
