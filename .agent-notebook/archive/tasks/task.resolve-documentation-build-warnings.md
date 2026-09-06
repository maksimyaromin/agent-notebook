---
id: task.resolve-documentation-build-warnings
type: task
state: closed
title: Resolve documentation build warnings
by: Maksim Yaromin
from: task.write-the-readme-and-book-for-engineers
link: note note.report-resolve-documentation-build
created: 2026-09-05
updated: 2026-09-06
closed: 2026-09-06
---

The documentation build completes and link checks pass, but reports an oversized JavaScript chunk and Starlight warnings for the absent i18n collection and 404 entry. Trace the build configuration and bundled integrations; remove the causes without suppressing useful warnings. Verify with pnpm docs:check.
- 2026-09-06 Maksim Yaromin: Traced the three warnings. The chunk is mermaid's core, 662 kB minified, loaded by astro-mermaid through a dynamic import only on a page with a diagram; the fences stay because GitHub renders them, so the chunk is inherent and the Vite limit now sits at 700 kB, just above it, so growth still warns. The i18n warning: Starlight reads the optional i18n collection unconditionally and silences console.warn around the read, but Astro 7 warns through its logger, so the silence misses; a declared but empty collection warns as well, so the collection is one file, src/content/i18n/en.yml, an empty mapping. The 404 warning: Starlight's route asks the docs collection for a 404 entry; a docs/404.md removes that warning and raises a route conflict with Starlight's own 404 route instead, so the theme's route is off (disable404Route) and the page is src/pages/404.astro through StarlightPage, built as 404.html at the site root. The build reports nothing; docs:check green.
- 2026-09-06 Maksim Yaromin: Smoke check: the largest chunk is the grammar tooling mermaid's diagram parsers share, not mermaid's core, which is 94 kB; the comment and the report say so now. Two comments restated a fact that has its home elsewhere and are down to their role clause.
