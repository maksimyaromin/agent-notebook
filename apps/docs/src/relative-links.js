import { relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

/**
 * Rewrites a relative link between two pages of the book into the route the site serves it at.
 *
 * The book is written so that GitHub renders it too, which means a cross-page link is written as the target's path on
 * disk: `./failures.md`, `../reference/api.md`. Neither Astro nor Starlight touches such a link, so without this it
 * reaches the built HTML unchanged and answers 404 on every page of the site. `pnpm validate:docs` holds authors to
 * that form, so this is where the two spellings meet.
 *
 * @see https://docs.astro.build/en/guides/markdown-content/
 */

const docsRoot = fileURLToPath(new URL('../../../docs/', import.meta.url))

const hasScheme = /^[a-z][a-z\d+.-]*:/i

/** The route a page is served at, derived the way `tools/docs/validate.ts` derives a sidebar slug. */
function routeOf(path) {
  const slug = relative(docsRoot, path)
    .split(sep)
    .join('/')
    .replace(/\.md$/, '')
    .replace(/(^|\/)index$/, '')

  return slug === '' ? '/' : `/${slug}/`
}

export function relativeMarkdownLinks() {
  return {
    name: 'relative-markdown-links',
    element: {
      filter: ['a'],
      visit(node, ctx) {
        const href = node.properties?.href

        if (typeof href !== 'string' || ctx.fileURL === undefined) {
          return
        }

        // A scheme names another origin, a leading slash names a route already, and a bare fragment stays on the page.
        if (hasScheme.test(href) || href.startsWith('/') || href.startsWith('#')) {
          return
        }

        const hash = href.indexOf('#')
        const target = hash === -1 ? href : href.slice(0, hash)
        const fragment = hash === -1 ? '' : href.slice(hash)

        if (!target.endsWith('.md')) {
          return
        }

        const resolved = fileURLToPath(new URL(target, ctx.fileURL))

        // Every page of the book lives under docs/, and a link out of that tree has no route to be rewritten into.
        // The same link fails `pnpm validate:docs`, so reaching here means the two checks disagree.
        if (!resolved.startsWith(docsRoot)) {
          throw new Error(`${href} in ${fileURLToPath(ctx.fileURL)} points outside ${docsRoot}`)
        }

        ctx.setProperty(node, 'href', routeOf(resolved) + fragment)
      },
    },
  }
}
