---
id: decision.the-local-date-is-worth-its-dependency
type: decision
state: active
kind: rule
title: The local date is worth its dependency
by: Maksim Yaromin
from: question.should-anb-resolve-the-local-date
tags: perf
created: 2026-08-30
updated: 2026-08-30
---

Measured on this machine: `anb status` on an empty notebook is 3.26 ms wall with jiff and 2.42 ms with the date replaced by a constant — so one `Zoned::now().date()` costs ~0.83 ms, the majority of what anb itself does once the process floor is subtracted, plus 198 KB of a 1.43 MB binary. It stays. The cheaper alternatives each cost correctness or reach: trimming jiff's features drops the tz database and makes `TZ=Pacific/Midway anb add` write tomorrow's date, and reading /etc/localtime by hand saves ~0.7 ms but is Unix-only, which the planned npm distribution cannot assume. Against the notebook itself — a thousand live records cost 40 ms for a single command — 0.83 ms an invocation is not where this tool's time goes. Revisit only on new evidence: a platform matrix that makes binary size decisive, or an upstream fix to jiff's scan of every zone in /usr/share/zoneinfo at first use.
