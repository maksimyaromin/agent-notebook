---
title: Drawing the notebook
description: 'The graph anb serves as data, and the atlas skill that draws it into one page where a reader decides by pointing.'
---

The CLI serves the notebook as data and draws nothing. Whoever wants a picture builds one from the data, in the shape the question needs; a page for "pick the next Task" is not the page for "show me the whole notebook", and only the asker knows which one they wanted.

## The graph

`anb graph` answers with the records and the edges between them, on any slice:

| Question | Command |
|---|---|
| the whole notebook, archive included | `anb --json graph --archive --full --all` |
| one epic's branch | `anb --json graph --for <hub> --full --all` |
| what can start now | `anb --json graph --ready --full --all` |
| around one record | `anb --json graph --focus <id> --depth 2 --full --all` |
| one kind of record | `anb --json graph --type task --full --all` |

The JSON carries `v`, the format version; `slice`, every narrowing that made the document; `nodes`, each with `id`, `type`, `state`, `ready`, `archived`, `degree`, `created`, `title`, a Task's `priority` when it has one, a hub's `epic` progress, and under `--full` the envelope and body; and `edges` of three kinds. `waits` runs out of the record that must settle first into the one waiting on it; `born` runs out of the origin into the record born from it; `mentions` runs the way it was written. The plain text is bounded like every listing; the JSON never is, because a drawing made from some of the edges is a picture of a notebook that does not exist.

## The atlas skill

The `anb-atlas` skill, installed by `anb setup` beside the `anb` skill, carries what a good page needs: one command as the whole input, printed at the bottom of the page with the slice it made; one self-contained file with the data embedded, no library and nothing fetched; one visual channel per fact (kind by hue, settled by fill, archived by opacity, degree by radius, relation by stroke); a legend that is also the filter; finished work drawn distinct and hubs carrying `closed/total`; a record that opens beside the map in a side panel, never in a covering modal; and the layout lessons that cost the most to learn, from pointer capture to label placement.

The page is where the reader decides, and the CLI is where the notebook changes. Comments made on the page are addressed to a record id or to the slice, come back to the agent as one batch, and become one command each, run through `anb` and reported per comment. Nothing writes the notebook from the page.

Read the skill itself in the repository under `.agents/skills/anb-atlas/`, or install it into any project with `anb setup`.
