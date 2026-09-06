// @ts-check
import starlight from '@astrojs/starlight'
import mermaid from 'astro-mermaid'
import { defineConfig } from 'astro/config'
import starlightLlmsTxt from 'starlight-llms-txt'

import { relativeMarkdownLinks } from './src/relative-links.js'
import { sidebar } from './src/sidebar.js'
import { site } from './src/site.js'

const repository = 'https://github.com/maksimyaromin/agent-notebook'

/**
 * Adds the link rewriter to the Markdown pipeline Astro assembled, so a link
 * written as a path on disk (`../reference/records.md`, which GitHub
 * renders) becomes the route the site serves.
 *
 * @see https://docs.astro.build/en/reference/configuration-reference/#markdownprocessor
 * @type {import('astro').AstroIntegration}
 */
const linkRewriting = {
  name: 'relative-markdown-links',
  hooks: {
    'astro:config:setup': ({ config }) => {
      const { options } = /** @type {{ options: { hastPlugins: unknown[] } }} */ (
        config.markdown.processor
      )

      options.hastPlugins.push(relativeMarkdownLinks)
    },
  },
}

/** @param {string} slug */
const pageUrl = (slug) => new URL(`/${slug}/`, site).href

export default defineConfig({
  site,
  integrations: [
    mermaid({ theme: 'neutral', autoTheme: true }),
    linkRewriting,
    starlight({
      // Starlight runs its own Markdown transforms only on files under the
      // collection directory it expects; the book lives in `docs/` instead.
      markdown: { processedDirs: ['../../docs'] },
      title: 'agent-notebook',
      description:
        'A dependable CLI for agent working memory, with skills you can make your own.',
      logo: { src: './src/assets/mark.svg', replacesTitle: false },
      // Starlight emits one link for the file named here. The rest are the
      // fallbacks it does not emit: a raster icon for the browsers that take
      // one over an SVG, and the icon a phone uses on its home screen.
      favicon: '/favicon.svg',
      head: [
        { tag: 'link', attrs: { rel: 'icon', href: '/favicon-32.png', sizes: '32x32' } },
        { tag: 'link', attrs: { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' } },
      ],
      social: [{ icon: 'github', label: 'GitHub', href: repository }],
      editLink: { baseUrl: `${repository}/edit/main/` },
      routeMiddleware: './src/route-data.js',
      lastUpdated: true,
      plugins: [
        starlightLlmsTxt({
          projectName: 'agent-notebook',
          description:
            'agent-notebook gives coding agents a deterministic CLI for working memory. Skills define the workflow and can be rewritten or replaced; sharing the Markdown records through git is optional.',
          details:
            'Tasks, Decisions, Notes and Questions have explicit lifecycles and relationships. Commands check changes and report results or refusals in bounded text or JSON. The supplied skills cover session handoffs and interactive maps. Notebook location, git sharing and personal knowledge across projects are configurable choices.',
          optionalLinks: [
            {
              label: 'Commands',
              url: pageUrl('reference/commands'),
              description: 'Every command with its arguments and flags, rendered from the binary.',
            },
            {
              label: 'Records and files',
              url: pageUrl('reference/records'),
              description: 'The four record types, their states, the envelope keys and the file format.',
            },
            {
              label: 'Replies',
              url: pageUrl('reference/replies'),
              description: 'The reply contract: ok lines, tables, bounds, JSON, refusals and try lines.',
            },
            {
              label: 'Refusals and findings',
              url: pageUrl('reference/refusals'),
              description: 'Every refusal code with its example and repair, and the check findings by severity.',
            },
          ],
        }),
      ],
      sidebar,
    }),
  ],
})
