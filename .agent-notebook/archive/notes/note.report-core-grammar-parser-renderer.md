---
id: note.report-core-grammar-parser-renderer
type: note
state: retired
title: Report: Core: grammar parser + renderer
by: Maksim Yaromin
via: claude-code
from: task.core-grammar-parser-renderer
created: 2026-08-29
updated: 2026-08-29
---

# c1 — Core: grammar parser + renderer — completion report (2026-08-26)

Delivered in `crates/anb-core`: `src/finding.rs` (Finding/FindingCode/Severity, the S1 finding codes), `src/grammar.rs` (`RecordFile::parse/render/normalize/placement_findings`; two-pass parse — structural `parse_structure`/`scan_envelope`, then semantic `check_fields`; the field table as `required`/`optional`/`repeatable` constructors; uniform per-form validators), `tests/corpus.rs` over `testdata/corpus` (52 cases with `.findings`/`.normalized` sidecars, `.gitattributes -text`), `tests/grammar_props.rs` (render∘parse identity on any input, normalize idempotence, generated well-formed records).

Acceptance criteria: render(parse(x)) == x byte-exact on any input (property-tested); normalize idempotent (property + corpus); negative corpus green — every almost-valid file yields named findings, bytes never rewritten while invalid. Gate (`./scripts/check.sh`) green; deliberate can-it-fail red runs performed on all three test layers.

Owner rulings folded into code and `spec-anb-format.md`: slug minimum one character; id-prefix-vs-type coherence as `bad-id`; `no-final-newline` only when the file ends inside the envelope; `bad-envelope-line` is an error; a present field must satisfy its form (`by` alone admits empty); `crlf` once per file; `updated` not required on read. `not-utf8`/UTF-16 deferred to the fs adapter with the LLM-transparency duty recorded on task l3.

Deferred by design: mutation splicing and `already: true` (c2), graph findings (c2/c3), config parsing (first caller's ticket).
