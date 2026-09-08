---
id: decision.one-narrowing-for-every-read
type: decision
state: active
kind: shape
title: One narrowing for every read: a listing answers with what its filter admits
by: Maksim Yaromin
via: claude-code
from: task.read-side-complexity-revision
tags: cli, read-side
created: 2026-09-08
updated: 2026-09-08
---

Every read that prints a row takes one Filter: type, kind, tag, for (an epic's scope), by (an identity; --mine is by with the caller's name, --team is nobody, and the config key scope decides when the call names none), match (a text over id, title, tags, people and body) and archive. Each flag is a predicate over the same notebook, two flags ask for the intersection, and a filter narrows what is shown and never what is read, so no narrowing can free a blocked Task or change a hub's count. A flag means the same on every verb that takes it, and a verb takes only the flags it can answer: list and graph take the whole Filter; ready takes for, tag, match and whose, and refuses type, kind and archive on the command line, since a queue of live Tasks could only answer them with nothing more; status takes by, mine and team.

The duplicates the inventory found, and their rulings: graph --ready is deleted, because a ready Task waits on nothing live, so a graph of ready Tasks alone is a listing drawn as tiles, and every node still carries ready for a page to key on; graph --type stays and list gains it, since both read the one Filter; overview is deleted, because list --all is the page and list --type <t> a section of it; search is deleted, because a substring is one more predicate, list --match, with --archive reaching history as search once did by default; --mine stays as the spelling of --by with the identity filled in, since a hint that lifts a bound has to carry a name that runs for whoever types it.

Status is the work: active Tasks with the first one's last log line, review, held, the ready queue, the open Questions, and one line counting Debt. Decisions and Notes never reach it: a rule is read before the work it binds, by list --type decision --kind rule, not pushed into every session's opening, so a notebook of knowledge alone is quiet. Epics have no section: ready --for <hub> is an epic's queue, list --for <hub> --archive its membership, and a hub's graph node carries closed/total and next. anb debt is the read the count line points at.

The alternative weighed was one flag set on every listing whatever it can answer; it was rejected because a flag a verb cannot answer is a question the reference has to explain away, and a queue offered --archive reads as a queue that could hold history.
