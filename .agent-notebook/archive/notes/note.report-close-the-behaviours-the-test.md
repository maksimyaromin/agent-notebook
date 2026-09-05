---
id: note.report-close-the-behaviours-the-test
type: note
state: retired
title: Report: Close the behaviours the test audit proved unguarded
by: Maksim Yaromin
from: task.close-the-behaviours-the-test-audit
created: 2026-09-02
updated: 2026-09-05
---

# Report: closing the behaviours the test audit proved unguarded

Task: `task.close-the-behaviours-the-test-audit`. Date: 2026-09-02. Gate (`./scripts/check.sh`) green at hand-over: fmt, clippy `-D warnings`, 612 tests, 1 doctest, docs with `-D warnings`.

## 1. What the mutations showed at HEAD, before any change

Each of the six mutations the audit named was applied to the source at HEAD and the whole suite run, then the source restored. Four were still green, one was already red, and one is a generator gap no mutation can express.

| # | Mutation applied | Suite at HEAD |
| --- | --- | --- |
| 1 | `ready_for` and `list_for` fold over the scoped subset with a resolver built from that subset | green |
| 2 | `guard_destination_free` reads `NotUtf8` at the destination as free (`return Ok(())`) | green |
| 3 | `note_links` without its dedup filter | green |
| 4 | `view`'s `archived: true` line deleted from the text rendering; and, the other way, printed on every record | red both ways: `viewing_an_archived_record_names_it_as_history` catches the deletion, the exact `view` snapshot `view_prints_the_envelope_the_body_and_the_blocks` catches the line on a live record |
| 5 | the `add "<title>" --id` retry offered for a dangling reference of any type | green |
| 6 | the property generator never emits `blocked-by` | not a source mutation; closed by generating it, see §3 |

Item 4 was closed by a test that landed after the audit was written, so no test was added for it.

## 2. The tests that close 1, 2, 3 and 5

Each is one situation, asserted through the public verb, with expected values derived from the notebook's rules rather than from running the code. Each was proven red under its mutation after it was written, and green with the mutation restored.

- **1 — `a_scoped_row_is_resolved_against_the_whole_notebook`** (`crates/anb-core/tests/notebook/dependencies.rs`, mod `epics`). An epic with one child born inside it also waits on `task.adopted`, which was born from `task.elsewhere`, a live Task outside the scope. `list_for` shows `task.adopted` as `open` beside the child and the hub, and `ready_for` lists the adopted Task and the child. Under the mutation the Origin looks dangling to a scope-local resolver, the row is excluded, and the state reads `invalid`. The discriminator is the Origin because a child's blockers are always inside the scope by the scope walk, so no blocker can lie outside it; the Origin is the one reference a scoped record can carry to a record the scope leaves out.
- **2 — `a_live_record_does_not_move_onto_unreadable_bytes`** (`crates/anb-core/tests/notebook/check.rs`, mod `unreadable_files`). A closed live `task.demo` and bytes no parse can read at `archive/tasks/task.demo.md`. `archive` refuses with `DuplicateId` naming the archive path as holder, and the live file keeps its bytes. The classification mirrors the restore side, where `an_unreadable_live_holder_refuses_the_move` already expects `DuplicateId` naming the live path. The existing `an_unreadable_archived_copy_refuses_the_move_as_an_invalid_record` covers the replay path only (no live source), which is why mutation 2 slipped past it.
- **3 — `a_report_linked_twice_is_carried_once`** (`crates/anb-core/tests/notebook/archive.rs`). A closed Task whose envelope names `link: note note.report` twice. `archive` succeeds and `carried` names the report once; the success alone proves the cascade completed, so the carried list is the whole guard. Under the mutation the cascade files the report twice and the second move fails on the source the first removed.
- **5 — `a_dangling_reference_to_another_type_offers_no_creating_command`** (`crates/anb/tests/cli.rs`, mod `knowledge_replies`). `ask "A doubt" --from note.ghost` is refused with `dangling-ref` and offers `anb list` alone. Under the mutation a `try: anb add "<title>" --id note.ghost` line appears, a command `add` cannot run because it mints Tasks alone.

## 3. The property generator (6)

`generated_task` in `crates/anb-core/tests/notebook_props.rs` now emits `blocked-by: task.blocker` on half of the generated tasks, at the canonical field position the tool itself would write it, and answers `(text, carries_edge)`. The replay property keeps its two promises unchanged. A second property over the same generator, `an_edge_verb_answers_for_the_edge_the_file_carries`, holds the two edge verbs to the file: `block`'s answer is `already` exactly when the edge stands, `unblock`'s exactly when it does not. Without it the generated erase path had no teeth: an `unblock` that fails to see the edge under a quirk answers `already: true` twice and leaves every byte alone, which replay identity cannot tell from a correct replay.

Proven red with a mutation that hides the edge on a held task (`edge_exists` answering false when a `hold:` line stands); no unit test combines a hold with an edge, and both properties caught it on the first run, the replay one because a `block` that cannot see the edge appends a second copy on every call. The acceptance wording, a test that fails under the named mutation, has no literal form for this item: the mutation was in the generator, and reverting it leaves every property green by construction; the hold mutation is the nearest proof. The proptest regression seed that run wrote was discarded, since it records a mutation, not a defect.

## 4. The doctest finding

`cargo test --doc` ran over zero examples in both crates. One doctest now sits at the anb-core crate root (`crates/anb-core/src/lib.rs`): a host handing a `Notebook` a `MemoryStorage` and the day's date, minting a Task, starting it, and reading the state back. The expected id `task.parse-the-fences` derives from the minting rule (type word, dot, the title lowercased with non-alphanumerics folded to hyphens). It is the executable form of the crate's own claim that all data flows through the Storage seam fed by the host, and it is the first thing a host author reads. Proven red by changing the expected id. The shell crate still runs zero doctests, on purpose: its library target exists so its own integration tests can reach the binary's modules, and nobody embeds it.

## 5. Review

The mandatory review ran on Opus 5 against the frozen diff, with every skill loaded. It traced each mutation through the production code by hand, confirmed the four HEAD-green claims and the two already-red directions of item 4, ran the gate, the doctests and rustdoc with warnings denied, and found nothing at must-fix. Ten findings, all taken:

1. The `already`-follows-the-file assertion sat inside a property named for replays and bystanders, so the name no longer said what red meant. Moved into its own property over the same generator, and the file header keeps its two original promises.
2. The assertion's message named an input, not the promise. Now `an edge verb answers for the edge the file carries`.
3. The generator doc said "an edge on the blocker"; the line lands on the dependent. Now "onto the blocker".
4. The duplicate-link test asserted two `is_ok()` reads the `unwrap` already proved, one of them about the Task rather than the report. Dropped; the carried list is the guard.
5. The unreadable-destination test's name did not separate it from its sibling, which pins the opposite classification on the same path: the discriminator is whether a live record still stands. Renamed `a_live_record_does_not_move_onto_unreadable_bytes`.
6. Its doc claimed the bytes are somebody's only copy, which nothing in the test observes. Now states what the test proves: the move is refused and the live record stays.
7. The scope fixture's `task.epic` was no epic (no child born inside it), inside a module named `epics`. One genuine child added; the adopted Task sits beside it.
8. The doctest reached `NotebookError` by full path while importing its siblings, and repeated the date. Imported, and the date bound once as `today`, which also shows the contract the prose claims: the date comes from the host.
9. The shell crate still contributes no doctest. Left out on purpose, stated in §4.
10. A pre-existing gap the new test exposed: `archive`'s `# Errors` doc named `DuplicateId` only for a report that cannot be filed, never for the record's own destination. One clause added in `crates/anb-core/src/notebook.rs`.

After the fixes: gate green, all six mutations red again, the tell-string sweep over the added lines clean.

## 6. Files changed

`crates/anb-core/src/lib.rs` (doctest), `crates/anb-core/src/notebook.rs` (one doc comment), `crates/anb-core/tests/notebook/archive.rs`, `crates/anb-core/tests/notebook/check.rs`, `crates/anb-core/tests/notebook/dependencies.rs`, `crates/anb-core/tests/notebook_props.rs`, `crates/anb/tests/cli.rs`. No production logic changed.
