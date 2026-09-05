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
      title: 'agent notebooks',
      description:
        'A project’s working memory as typed records in plain files, read and written by any coding agent through one CLI.',
      favicon: '/favicon.svg',
      social: [{ icon: 'github', label: 'GitHub', href: repository }],
      editLink: { baseUrl: `${repository}/edit/main/` },
      routeMiddleware: './src/route-data.js',
      lastUpdated: true,
      plugins: [
        starlightLlmsTxt({
          projectName: 'agent notebooks',
          description:
            'A CLI named anb keeps a project’s working memory as typed records with lifecycles, in markdown files committed with the code, for any coding agent to read and write.',
          details:
            'Four record types (Task, Decision, Note, Question), one file each under .agent-notebook/, every change through a command that answers in a few lines of plain text or JSON; a Status the agent starts from, a dispatch queue, hubs for epics, proofs on close, and a skill the binary renders from itself.',
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
