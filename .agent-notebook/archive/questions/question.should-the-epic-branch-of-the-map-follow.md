---
id: question.should-the-epic-branch-of-the-map-follow
type: question
state: closed
title: Should the epic branch of the map follow blockers as deep as the queue does?
by: Maksim Yaromin
from: task.graph-emit-html
reason: One scope rule for every surface: ready --for, list --for and graph --for agree on what an epic contains, and the wide picture is truthful, since the v1 hub waits on nearly everything. The narrow picture already exists as graph --focus <hub> --depth 1.
created: 2026-08-30
updated: 2026-09-02
---

The map's --for slice uses the scope rule ready --for and list --for already share: the hub is blocked by it, or its Origin chain reaches it, both followed transitively. For a queue that is right — a blocker of a child must close before the child, so it is work the epic waits on however far down it sits. For a map it reads differently. On the live notebook, graph --for task.task-graph-visualization-for-fun draws 38 of 42 tasks, because that hub sits behind the release gate and the release gate waits on nearly everything. The epic branch is then the full map with four tiles missing. Option A (shipped): one scope rule for every surface. A slice that disagreed with the queue about what an epic contains would be worse than a wide picture. Option B: the map follows Origin to any depth but blocked-by only from the hub itself, so the branch is what was born inside the epic plus what the hub directly waits on. That is the picture a reader means by 'this epic', and it leaves ready --for alone. Option C: a depth flag on the map, which puts the judgement on the caller and gives the reader nothing to learn once.
