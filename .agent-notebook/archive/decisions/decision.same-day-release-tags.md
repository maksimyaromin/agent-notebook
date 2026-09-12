---
id: decision.same-day-release-tags
type: decision
state: superseded
kind: rule
title: A release tag is its day, and a later release the same day appends a counter from 1
by: Maksim Yaromin
from: task.release-2026-09-06-1
superseded-by: decision.release-date-version
created: 2026-09-06
updated: 2026-09-12
---

The first release of a day is tagged vYYYY.MM.DD. Each further release on the same day appends .N, counting from 1: v2026.09.06 is the day's first release and v2026.09.06.1 its second. The bare tag stands for a zero that is never written. Several releases a day are the normal pace of this project, and the tag orders them without a time in its name. The Release workflow refuses a tag of any other shape before it creates a release or publishes a package.
