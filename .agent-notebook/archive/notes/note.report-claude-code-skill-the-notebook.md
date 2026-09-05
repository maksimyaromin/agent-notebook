---
id: note.report-claude-code-skill-the-notebook
type: note
state: retired
title: Report: Claude Code skill: the notebook drawn and decided on in one page
by: Maksim Yaromin
from: task.claude-code-skill-the-notebook-drawn-and
created: 2026-09-05
updated: 2026-09-05
---

# The atlas skill: the notebook drawn and decided on in one page (2026-09-05)

Report for task.claude-code-skill-the-notebook-drawn-and. The skill teaches an agent to draw a notebook from `anb graph --json` into one self-contained page and to turn the reader's comments on that page back into commands.

## What shipped

```
.agents/skills/anb-atlas/
  SKILL.md                       the method, read once per session
  references/
    drawing.md                   what the data carries, the encoding, layout, the five lessons
    intent-loop.md               how comments are addressed, return as a batch, become commands
```

The skill is written by hand and committed, so Codex and Pi read it from this repository directly. The binary embeds the three files with `include_str!` and `anb setup` installs them beside the rendered `anb` skill into `.claude/skills/anb-atlas/` and `.agents/skills/anb-atlas/` of any project, under the same rules: a file whose frontmatter lost the mark is the user's and is left alone; `--remove` takes out the files and the directories they alone filled. Because setup now installs a rendered skill and a hand-written one by the same rule, the mark says what is true of both: `managed-by: anb`.

What the skill carries from the worked reference page and the maintainer's requirements:

- The founding split. The CLI serves the graph as data and draws nothing; the page is generated per question, and the picture belongs to whoever asked for it. One command is the whole input, printed at the bottom of the page with the slice it made, because a drawing that does not say what it left out reads as a notebook that holds nothing else.
- What the data carries, as the binary serves it: `v`, `slice`, `nodes`, `edges`; a node's fields including `epic` progress and the three-valued `ready`; the body as `lines` and `head`, whole under `--all`; the three edge kinds and their directions.
- The encoding, one channel per fact, in a table whose last column is the reason; the rail that is both legend and filter.
- Layout rules and the five lessons that cost the most: the pointer capture that steals the click, the synthetic click that is not a click, aim as a constant hit circle with the name as target, labels reserved in the layout rather than repaired, a hub's pull shared across its edges.
- The intent loop. The page is where the reader decides and the CLI is where the notebook changes. Comments are addressed to a record id or to the slice, return as one batch (a host's comment threads, or the page's own drawer exporting plain `id: instruction` lines), and become one command each, read against the record as it is now, run, and reported per comment. Nothing writes the notebook from the page, and a batch is never executed twice.
- The side panel beside the map, never a modal over it; finished work drawn distinct (settled outlined, archived faded, hubs with `closed/total`).
- What it must not inherit: a vendored library, or styling decided in a repository.

The skill follows the layout and the writing rules settled earlier in the marathon: a "Use when" description, the standard sections, references with contents lists, and prose that reads plainly.

## Tests

Unit: every atlas file carries the mark; `SKILL.md` links each reference where it lies. End to end: setup installs both skills (18 file lines), re-runs idle, removes them with their directories and never above a skill directory; a file the user made theirs is left alone. The drift check in CI still covers the rendered skill; the atlas cannot drift, since the binary embeds the committed files.

## Verified live

Two runs of the real Claude Code binary in a scratch project holding a copy of this notebook, with both skills installed by `anb setup` and the `anb` binary on the path.

Discovery: asked to list its skills and say when it would use each, and which one it would load for "draw me this notebook as a page I can comment on", Claude Code named `anb-atlas` for the drawing request and described it from the frontmatter alone as "drawing or mapping the notebook and deciding by pointing at a picture", and named `anb` for continuing, picking, recording and closing notebook work.

Drawing: asked to draw the notebook as a page it could be commented on, following the skill, and told not to open a browser or change the notebook, a fresh session produced one file of 424 KB. Checked by grep and by loading it in Chrome from a `file://` URL: the data came from `anb --json graph --archive --full --all`, embedded whole (122 records, 265 relations), with the command and the slice printed at the bottom; no external script, stylesheet or fetch; the rail as legend and filter with counts per key; hubs with `closed/total`; a web and a ranked layout; search; a side panel; a comments drawer that keeps comments beside a record id or `*` and exports them as `id: text` lines, under the footer line "this page changes nothing in the notebook: comments leave it as lines for anb". The notebook copy was byte for byte unchanged afterwards. The session reported what it left out honestly: no verification with a real pointer, because it was told not to open a browser. Two traps on the way, for whoever runs this next: `claude -p` needs `--permission-mode acceptEdits` and an `--allowedTools` list to write files and run `anb`, and `--allowedTools` is variadic, so the prompt has to arrive on stdin or it is swallowed as a tool name.

## Review

One review pass, against the code, the binary's own graph output and the texts: no must-fix. Five should-fix, all taken. The drawing reference under-described the data in three places: a hub's `epic` also carries `id` and `next`, the record to work on next inside it (now stated, and `SKILL.md` names `next` too); `priority` appears only on a Task that has one, like `ready`, and the text listed it as if every node carried it; the direction rule for edges explained `waits` and left `born` unexplained (each kind now has its own sentence). Three subjectless fragments broke the writing rule and are sentences now. The "made theirs, left alone" behaviour and the installable set had tests only through the rendered skill; the test now runs over a file of each skill, and a unit test pins the two skills setup installs. The check also noted that both references are under a hundred lines and carried contents lists anyway; the lists are gone, since a four-item table of contents on a sixty-line file is noise.
