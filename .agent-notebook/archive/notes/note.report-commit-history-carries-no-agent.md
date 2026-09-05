---
id: note.report-commit-history-carries-no-agent
type: note
state: retired
title: Report: Commit history carries no agent attribution before the repository goes public
by: Maksim Yaromin
from: task.commit-history-carries-no-agent
created: 2026-09-05
updated: 2026-09-05
---

# The history carries no agent attribution (2026-09-05)

Report for task.commit-history-carries-no-agent. The owner's ruling: this is their code and their ownership, and no commit in the public history names an agent as co-author or generator.

## What shipped

Three files under `.tmp/release/`, deliberately uncommitted since the commit that added them would be part of the history they clean, and an instruction beside them (`.tmp/release/README.md`).

`strip-attribution.pl` is the one transform, shared by the rewrite and the verifier: a message in, the same message out minus exactly four kinds of line, `Co-Authored-By: Claude …`, `Claude-Session: …`, the `Generated with [Claude Code]` line, and a bare `https://claude.ai/code/session_…` line. A message nothing was deleted from passes through byte for byte; one that lost lines loses the blank lines the deletion left at its end and ends with a newline exactly when the original did.

`scrub-history.sh <clone>` rewrites every branch of a clone in place, with `git filter-repo` when installed and `git filter-branch` otherwise, both applying the same transform and both leaving the same commits; it adds back the remote filter-repo drops and deletes the backup refs and reflog filter-branch keeps. `verify-scrub.sh <original> <clone>` is the proof the owner runs before any push: the same number of commits on main, every tree identical in order, every author and committer with their dates identical, every message byte-equal to the original passed through the transform, no attribution line anywhere in the clone including other refs and the reflog, and the old main commit gone.

## Verified

On a fresh clone of origin today: main at `71f2199`, 87 commits, 80 attribution lines (27 naming Claude Opus 5 with 1M context, 27 Claude Fable 5, 6 Claude Opus 5, 20 session links). After the scrub with either backend: main at `4b1f041`, the verifier answered ok, zero lines left anywhere in the clone.

## The pull requests

Merged pull requests #5 and #6 carried the "Generated with" line and a session link in their descriptions on GitHub, which a rewrite of git history does not reach. Both bodies were edited today to remove exactly those lines; the originals are kept under `.tmp/release/pr-bodies-before/`. No other merged pull request carried one, and since the ruling none has been written: the conventions in AGENTS.md forbid it and every pull request since #7 is clean.

## The owner's hand

The push is the owner's, from the scrubbed clone, with a lease on the main they cloned so that a push in between is refused rather than overwritten:

```sh
git ls-remote --heads origin
git clone git@github.com:maksimyaromin/agent-notebook.git /tmp/anb-scrub
bash .tmp/release/scrub-history.sh /tmp/anb-scrub
bash .tmp/release/verify-scrub.sh . /tmp/anb-scrub
git -C /tmp/anb-scrub push --force-with-lease=main:<the main sha you cloned> origin main
```

Then `git fetch origin && git reset --hard origin/main` in every clone, the working copy first, with a clean tree. There are no tags; the first tag goes on after the scrub, since a tag pushed before it would pin the old history. The first command lists the branches on origin: main must be the only one, since any other branch keeps the old commits reachable, and a fix branch that was open for a few minutes today showed exactly that.

## Smoke check

Sonnet 5, once, exercising both backends on fresh clones (it installed `git-filter-repo` to test the preferred path). Three must-fix, all taken: the verifier used bash process substitution under a `sh` shebang and could not run as documented (both scripts are bash now and the instruction says so); the instruction claimed main was the only branch on origin while a fix branch was open at that moment (the run now begins by listing the branches, and the instruction says what a second branch means); `git filter-repo` drops the `origin` remote, so the documented push would have failed on the preferred path (the script adds it back). Three should-fix, all taken: the two backends disagreed by one byte on messages without a final newline, and the verifier compared through command substitution and could not see it (one Perl transform now serves the rewrite, the fallback and the verifier, which compares raw bytes with a marker, and both backends give the same commit); the filter-branch fallback left the old history under `refs/original` and in the reflog (deleted and pruned, and the verifier now fails if the old main commit is still present). Every tree, author, committer and date held for every commit under both backends, and the rewrite reproduced the recorded hash.
