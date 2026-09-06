---
id: note.report-resolve-documentation-build
type: note
state: retired
title: Report: Resolve documentation build warnings
by: Maksim Yaromin
from: task.resolve-documentation-build-warnings
created: 2026-09-06
updated: 2026-09-06
---

# Documentation build warnings resolved

The documentation build reports nothing, and `pnpm docs:check` stays green with 18 pages reachable and every link resolving. Three warnings were traced to their causes.

The oversized chunk is the grammar tooling mermaid's diagram parsers share, 662 kB minified; the file named for mermaid's core is a seventh of that. astro-mermaid loads it through a dynamic import and only on a page that shows a diagram; the two pages with diagrams keep their fences because GitHub renders the same file. The chunk is inherent to that choice, so Vite's warning limit is set to 700 kB, just above the chunk, with the fact stated beside the setting. A chunk that grows past it warns again.

The absent i18n collection: Starlight reads its optional translations collection on every build and wraps the read in a console.warn silence, but Astro 7 warns through its own logger, so the silence misses. A collection that is declared but holds no file warns as well. The collection is now declared in content.config.ts with Starlight's loader and schema, and holds one file, src/content/i18n/en.yml, an empty mapping with the reason stated in a comment.

The missing 404 entry: Starlight's 404 route asks the docs collection for an entry named 404 and warns when there is none. A docs/404.md removes that warning and raises another, a route conflict between the collection page and Starlight's own 404 route. So Starlight's route is switched off with disable404Route, and the page is src/pages/404.astro rendered through StarlightPage: title Page not found, the splash template, a line saying nothing is at the address and a link to the front page. Astro builds it as 404.html at the site root, where Cloudflare Pages serves it for any address that names nothing. The book under docs/ is unchanged and the sidebar rule of the docs check keeps no exception.
