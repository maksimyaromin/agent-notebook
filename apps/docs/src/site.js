/**
 * The origin this book is served from.
 *
 * `astro.config.mjs` builds every canonical URL and the sitemap from it, and
 * `pnpm docs:check` resolves a link written against it back to the page on
 * disk, so a README link into the site is checked like a relative one.
 */
export const site = 'https://agent-notebook.supolka.dev'
