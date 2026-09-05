---
id: question.should-anb-resolve-the-local-date
type: question
state: closed
title: Should anb resolve the local date without jiff?
by: Maksim Yaromin
tags: perf
resolved-by: decision.the-local-date-is-worth-its-dependency
created: 2026-08-30
updated: 2026-08-30
---

jiff is linked for one expression — `Zoned::now().date()` — and costs 745 us per invocation (60% of what anb itself does on an empty notebook) plus 198 KB of the 1.43 MB binary. The cost is jiff walking all 599 zones of /usr/share/zoneinfo to build a name list before the first lookup; a hand read of /etc/localtime plus a TZif parse of the same file is 27 us. Trimming jiff's features is not the answer: without tzdb-zoneinfo the TZ name is ignored, and `TZ=Pacific/Midway anb add` writes tomorrow's date. Reading /etc/localtime directly saves 714 us and 33 KB but drops Windows and jiff's TZ heuristics. Decide this beside npm distribution, where the platform matrix is fixed, not by inertia.
