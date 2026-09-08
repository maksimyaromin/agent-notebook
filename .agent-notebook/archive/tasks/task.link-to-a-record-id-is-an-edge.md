---
id: task.link-to-a-record-id-is-an-edge
type: task
state: closed
title: A link to a record id is an edge the tool walks
by: Maksim Yaromin
via: claude-code
taken-by: Maksim Yaromin
link: issue https://github.com/maksimyaromin/agent-notebook/issues/64
link: note note.report-link-to-a-record-id-is-an-edge
priority: 1
created: 2026-09-08
updated: 2026-09-08
closed: 2026-09-08
---

A link whose target is a record id forms an edge carrying the link's kind, from the record that carries the link into the record it names. show lists the incoming ones as linked-by, graph draws them with the link's kind as the edge kind, list --for and graph --for reach the records that link the hub, --focus walks them, and --match finds a record by its link lines. The skill mentions the relationship where relations are taught and teaches no workflow around it.
- 2026-09-08 Maksim Yaromin/claude-code: Record::linked_records is the one home of the edge; kin_of and MembershipIndex follow it, so --for, --focus and the archived-kin walk reach it; show lists linked-by by kind then id; EdgeKind::Link carries the link's word and EdgeKind::word spells every edge once for both renderers; graph JSON reads v 4. --match over link lines was priced and dropped: git already ties records to code. Five tests, each proven red without the edge.
- 2026-09-08 Maksim Yaromin/claude-code: Smoke check: two holes, both closed. A link kind spelled waits, born or mentions is refused at write, owned as EdgeKind::DRAWN_WORDS; a record cannot link itself, refused by edit and drawn by nothing. One test, proven red with each guard cut. Both gates green.
