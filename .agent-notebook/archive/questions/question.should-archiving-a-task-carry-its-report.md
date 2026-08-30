---
id: question.should-archiving-a-task-carry-its-report
type: question
state: routed
title: Should archiving a Task carry its report Note along?
by: Maksim Yaromin
from: task.cli-close-note-ingests-the-report-as-a-l
tags: cli
routed-to: decision.archiving-a-record-carries-its-reports
created: 2026-08-29
updated: 2026-08-30
---

close --note leaves the report Note live beside the task. Archiving the task moves only the task: the Note stays in notes/ as active, and check says nothing about the split. The retrofitted ten are archived-and-retired only because that was done by hand, so the first real close-then-archive cycle produces exactly the mixed state the retrofit does not show. Three shapes: archive carries records born from the archived one (a one-record verb becomes a cascade, and its idempotence story grows), a Check finding names an archived record whose report is still live (a new code for a housekeeping split), or the pairing stays a convention a reader keeps by hand. Born from the independent review of close --note.
