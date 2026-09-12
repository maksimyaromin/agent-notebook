---
title: Domain modeling
description: 'Give different roles a shared language for the system, grounded in scenarios and canonical sources.'
---

A shared domain language lets people and their agents reason about the same system. A glossary can define a package, an operation and a candidate. A model explains who approves a candidate, what publication changes and which rules must hold when an operation is executed. A `model` Note can keep that explanation when it has no suitable maintained home already.

Start from concrete scenarios and check them against the product, code and authoritative sources. Product can explain the intended outcome, support can supply failure cases, design can expose an ambiguous state, and engineering can identify which changes must stay consistent. They contribute to one account of the domain, not separate versions for each role.

Distinguish current behavior from a proposed design and state what supports that distinction. An active model Note means the explanation is maintained; it does not mean a proposal is approved or implemented. Authorship identifies a contributor, not the audience of a project rule.

## Choose the right amount of structure

Keep the smallest explanation that resolves a real ambiguity. It may be a few paragraphs in an existing model, not a new document. Explain responsibilities, vocabulary and the rules that connect the concepts. Add identity, consistency boundaries, allowed state changes or integration details when a scenario needs them. For a longer model, prepare its body in a file and add it from there:

```sh
anb add note "The operation library" --id note.operation-library --kind model --tag operation-library --via codex --body-file model.md
```

The example assumes `model.md` contains the explanation. Its explicit id makes later commands reproducible; ordinary additions can use the generated id returned by the CLI. `--body-file -` reads standard input instead. Read the existing Note before `anb edit note.operation-library --body-file model.md`, which replaces its entire body.

Separate models when different responsibilities or meanings make a single explanation misleading. For example, “published” might mean available for execution in the operation library but visible to a customer in another context. Explain the translation and the information each context supplies. A context map can help when several such boundaries matter; it is not required for every project. A source directory or class name alone does not establish a domain boundary.

## Separate definitions, relationships and choices

| Record | What it answers |
|---|---|
| Note `term` | What does this word mean in this context? |
| Note `model` | How do the concepts relate, and who owns their behavior? |
| Decision `rule` | Which agreed requirement must hold? |
| Decision `shape` | Why was this boundary or design chosen over the alternatives? |
| Note `spec` | What behavior should this change deliver? |

Choose a separate record only when that question needs independent retrieval or maintenance. These are possible homes, not a sequence of documents to create. A model can cite its governing Decisions, and relevant Tasks can cite the model, without keeping independently worded copies of the same rule.

Keep a definition in the model while it is only needed there. Create a separate term Note when it needs independent lookup or is reused elsewhere. If two contexts use the same word differently, qualify their definitions instead of forcing a common meaning.

## Make the model discoverable

The `model` kind identifies model Notes. A shared subject tag such as `operation-library` can also connect terms, Decisions and work in that context:

```sh
anb list --type note --kind model --tag operation-library
anb recall "operation library"
anb show note.operation-library --all
```

Knowledge retrieval is shared across authors by default, even when work reads follow one person's assignments. A manager and a developer should find the same applicable project rule. Use tags and links deliberately; sharing a tag does not prove agreement, conflict or a dependency.

Before changing domain behavior, read the relevant model and sources. Record an accepted replacement rule as a Decision when it needs to govern future work, and update affected explanations to point to it. A clarification can be an edit to the existing record. A refactor that preserves domain behavior does not need a new domain Decision.

## Use existing documentation

An existing glossary, specification or decision journal can remain authoritative. The notebook keeps the relevant conclusion, its scope and a link to that source. A research report may have one finding worth remembering; there is no need to copy the report or synchronize a second specification. Keep delivery commitments in the team's tracker and published explanations in their established documentation.

Preserve source links and distinguish drafts from accepted documents. When the code and a document disagree, record the discrepancy before assuming which should change. Private sources follow the notebook's [storage and sharing policy](customization.md); a shared record should explain its conclusion without requiring access to a private file.
