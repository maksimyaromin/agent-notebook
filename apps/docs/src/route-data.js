import { posix } from 'node:path'

import { defineRouteMiddleware } from '@astrojs/starlight/route-data'

/**
 * Points each page's "Edit page" link at the file the page was built from.
 *
 * Starlight forms that link by joining `editLink.baseUrl` with the entry's path as the loader reports it, and the
 * loader reports a path relative to this app. The book is two directories above, so the joined string climbs back out
 * with `../../` and the URL constructor collapses those segments together with the `edit/main` they climb through.
 * Rebasing the path on the repository root first leaves nothing to collapse.
 *
 * @see https://starlight.astro.build/guides/route-data/
 */

const editBase = 'https://github.com/maksimyaromin/agent-notebook/edit/main/'

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute
  const filePath = route.entry?.filePath

  if (typeof filePath !== 'string') {
    return
  }

  const inRepository = posix.normalize(`apps/docs/${filePath}`)

  // Starlight synthesises an entry for the 404 page, and its path names a file this repository does not carry. There
  // is nothing to edit, so the link goes rather than pointing at a page that does not exist.
  route.editUrl = inRepository.startsWith('docs/') ? new URL(inRepository, editBase) : undefined
})
