---
id: task.close-the-behaviours-the-test-audit
type: task
state: closed
title: Close the behaviours the test audit proved unguarded
by: Maksim Yaromin
tags: testing
link: note note.report-close-the-behaviours-the-test
priority: 3
created: 2026-08-30
updated: 2026-09-02
closed: 2026-09-02
---

Six mutations the suite does not catch, each confirmed green under the mutation: (1) a scope computed over the scoped subset with a scope-local resolver — the doc says a scope narrows what is shown, never what is read, and nothing observes it; (2) an unreadable archive destination read as free instead of occupied, so archive would overwrite unreadable history; (3) note_links dedup — a duplicated link: note line makes an archive cascade fail mid-way with a spurious not-found; (4) view's archived: line in text, so the text rendering cannot tell history from live work; (5) the add --id retry offered for a non-Task dangling reference, a try: line that cannot run; (6) the property generator never produces a task already carrying blocked-by, so unblock's erasing replay is never generated. Acceptance: each closed by a test that fails under the named mutation.
- 2026-08-31 Maksim Yaromin: CI review finding (2026-08-31): both crates carry zero doctests, so the gate's doctest step (cargo test --doc) and CI pass vacuously over 0 examples — the 'doctests' criterion is currently proof of nothing.
- 2026-09-02 Maksim Yaromin: Re-ran the six mutations against the suite at HEAD before writing anything: (1) scope-local resolver, (2) unreadable archive destination read as free, (3) note_links without dedup, (5) add --id offered for any dangling type were still green; (4) view's archived line was already red in both directions (viewing_an_archived_record_names_it_as_history catches its removal, the exact view snapshot catches printing it on a live record), so no test added for it. Closed (1) with a_scoped_row_is_resolved_against_the_whole_notebook, (2) with an_unreadable_file_at_the_archive_destination_holds_it, (3) with a_report_linked_twice_is_carried_once, (5) with a_dangling_reference_to_another_type_offers_no_creating_command, (6) by generating blocked-by in the property and holding block/unblock's first already to whether the file carries the edge. Doctest finding closed with one host example at the anb-core crate root. Every one red under its mutation, gate green.
- 2026-09-02 Maksim Yaromin: Opus 5 review: no must-fix, 3 should-fix, 7 nits, all taken — the edge assertion moved into its own property (an_edge_verb_answers_for_the_edge_the_file_carries), assertion message names the promise, generator doc says onto the blocker, redundant is_ok reads dropped, unreadable-destination test renamed a_live_record_does_not_move_onto_unreadable_bytes with its doc claiming only what it observes, the scope fixture's hub got a real child, the doctest imports NotebookError and binds today once, archive's Errors doc now names DuplicateId on the record's own destination. Shell crate left without a doctest on purpose. Gate green, all six mutations red again. Report at .tmp/docs/report-test-audit-closure.md.
