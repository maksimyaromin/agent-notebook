---
id: task.core-grammar-parser-renderer
type: task
state: closed
title: Core: grammar parser + renderer
by: Maksim Yaromin
via: claude-code
tags: core
link: note note.report-core-grammar-parser-renderer
blocked-by: task.spike-storage-format
blocked-by: task.rust-scaffold
created: 2026-08-24
updated: 2026-08-26
closed: 2026-08-26
---

render(parse(x)) === x on untouched input; normalize idempotent; negative corpus green - every almost-valid file yields a named Check finding, never a silent drop.

Progress 2026-08-26: implementation done and self-code-reviewed, at review pause (working tree, no Lavish per the new convention). Delivered: crates/anb-core/src/finding.rs (Finding with at/for_file constructors, FindingCode, Severity), crates/anb-core/src/grammar.rs (RecordFile::parse/render/normalize/placement_findings; two-pass parse: parse_structure/scan_envelope then check_fields; field table as required/optional/repeatable constructors; uniform form validators returning Option<reason>), tests/corpus.rs runner over testdata/corpus (51 cases, .findings/.normalized sidecars, .gitattributes -text), tests/grammar_props.rs (render-parse identity on any input, normalize idempotence, generated well-formed records). Gate green; can-it-fail red runs done on all three test layers (corpus re-proven after the rewrite). Owner feedback folded in: engineering skills applied per line, error messages rewritten to engineer register, mandatory self-review stage added to AGENTS.md, Lavish removed from coding-task reviews. Owner rulings 2026-08-26: all validation decisions approved as proposed; slug minimum relaxed to one character (code, spec section 6 regex, corpus case valid/id-single-char-slug.md, property generator updated); id-prefix-vs-type coherence, no-final-newline scoping, bad-envelope-line severity, empty-optional strictness, crlf-once, and updated-not-required-on-read recorded in spec-anb-format.md; not-utf8/UTF-16 stays deferred to the fs adapter with the LLM-transparency duty recorded on task l3. Gate green after all changes.
