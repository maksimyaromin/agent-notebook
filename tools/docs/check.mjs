/**
 * Documentation check, `pnpm docs:check`.
 *
 * Three rules over the book. The sidebar reaches every page under docs/, so
 * none is unreachable. Every relative link in a page resolves to a file, and
 * a link naming a heading finds one. And after the site is built, every
 * reference a built page makes to this site names a file the build wrote,
 * which is what a reader would otherwise meet as a 404.
 */
import { spawnSync } from 'node:child_process'
import { existsSync, globSync, readFileSync, statSync } from 'node:fs'
import { dirname, join, relative, resolve, sep } from 'node:path'
import process from 'node:process'

import { sidebar } from '../../apps/docs/src/sidebar.js'

const repoRoot = resolve(import.meta.dirname, '../..')
const docsRoot = resolve(repoRoot, 'docs')
const dist = resolve(repoRoot, 'apps/docs/dist')

const failures = []
const fail = (file, message) => failures.push(`${relative(repoRoot, file)}: ${message}`)

const pages = globSync('**/*.md', { cwd: docsRoot }).map((path) => path.split(sep).join('/'))
const slugOf = (page) => page.replace(/\.md$/, '').replace(/(^|\/)index$/, '')

// 1. The sidebar reaches every page.
const reached = new Set()
const walk = (items) => {
  for (const item of items) {
    if (item.slug !== undefined) reached.add(item.slug)
    if (item.items) walk(item.items)
  }
}
walk(sidebar)
for (const page of pages) {
  const slug = slugOf(page)
  if (slug !== '' && !reached.has(slug)) fail(join(docsRoot, page), 'no sidebar entry reaches this page')
}
for (const slug of reached) {
  if (!existsSync(join(docsRoot, `${slug}.md`)) && !existsSync(join(docsRoot, slug, 'index.md'))) {
    fail(join(repoRoot, 'apps/docs/src/sidebar.js'), `names ${slug}, which has no page`)
  }
}

// 2. Relative links resolve, headings included.
const hasScheme = /^[a-z][a-z\d+.-]*:/i
const linkPattern = /\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g
const headingsOf = (text) =>
  new Set(
    [...text.matchAll(/^#{1,6}\s+(.+?)\s*$/gm)].map((m) =>
      m[1]
        .toLowerCase()
        .replace(/`/g, '')
        .replace(/[^\p{L}\p{N}\s-]/gu, '')
        .trim()
        .replace(/\s+/g, '-'),
    ),
  )
for (const page of pages) {
  const file = join(docsRoot, page)
  const text = readFileSync(file, 'utf8')
  const withoutCode = text.replace(/^[ \t]*```[\s\S]*?^[ \t]*```/gm, '').replace(/`[^`\n]*`/g, '')
  for (const match of withoutCode.matchAll(linkPattern)) {
    const target = match[1]
    if (hasScheme.test(target) || target.startsWith('//')) continue
    const [path, fragment] = target.split('#')
    const resolved = path === '' ? file : resolve(dirname(file), path)
    if (!existsSync(resolved)) {
      fail(file, `link to ${target} names no file`)
      continue
    }
    if (fragment && statSync(resolved).isFile() && !headingsOf(readFileSync(resolved, 'utf8')).has(fragment)) {
      fail(file, `link to ${target} names no heading`)
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}

// 3. The built site references only files the build wrote.
const build = spawnSync('pnpm', ['docs:build'], { cwd: repoRoot, stdio: 'inherit' })
if (build.status !== 0) process.exit(build.status ?? 1)

const reference = /(?:href|src)="([^"]*)"/g
const leavesTheSite = (target) =>
  target === '' || target.startsWith('#') || target.startsWith('//') || hasScheme.test(target)
for (const html of globSync('**/*.html', { cwd: dist })) {
  const file = join(dist, html)
  for (const match of readFileSync(file, 'utf8').matchAll(reference)) {
    const target = match[1]
    if (leavesTheSite(target)) continue
    const path = decodeURIComponent(target.split(/[?#]/)[0])
    const absolute = path.startsWith('/') ? join(dist, path) : resolve(dirname(file), path)
    const candidates = [absolute, join(absolute, 'index.html')]
    if (!candidates.some((candidate) => existsSync(candidate) && statSync(candidate).isFile())) {
      fail(file, `references ${target}, which the build did not write`)
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join('\n'))
  process.exit(1)
}
console.log(`docs: ${pages.length} pages reachable, every link resolves, the built site is whole`)
