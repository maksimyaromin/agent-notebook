import { docsSchema } from '@astrojs/starlight/schema'
import { defineCollection } from 'astro:content'
import { glob } from 'astro/loaders'

// The book is the `docs/` tree of the repository, read where it lives. One
// copy means GitHub renders a page when someone browses the repository and
// this site renders the same file, and a relative link between two pages
// works in both.
export const collections = {
  docs: defineCollection({
    loader: glob({ base: '../../docs', pattern: '**/[^_]*.md' }),
    schema: docsSchema(),
  }),
}
