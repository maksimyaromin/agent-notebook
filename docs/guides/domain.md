---
title: Domain modeling
description: 'Use model Notes, terms and Decisions to describe a domain and keep it consistent with the work.'
---

A glossary can define a package, an operation and a candidate. It cannot by itself explain who approves a candidate, what publication changes or which rules must hold when an operation is executed. A Note of kind `model` describes these relationships.

The supplied skill helps an agent develop the model through concrete scenarios and check it against the code. It distinguishes existing behavior from a target design; an active model Note does not establish that the implementation already follows it.

## Choose the right amount of structure

Start with one model when the domain is small. Explain its purpose, responsibilities, vocabulary and the rules that connect its concepts. Add detail where it resolves an actual question: identity, aggregate boundaries, allowed state changes or information exchanged with another context.

For a project with several bounded contexts, keep a model Note for each context that needs its own language and responsibilities. A context-map Note cites those models and describes who supplies information to whom, what the contract means and where translation is needed. A source directory or class name alone does not establish a domain boundary.

## Separate definitions, relationships and choices

| Record | What it answers |
|---|---|
| Note `term` | What does this word mean in this context? |
| Note `model` | How do the concepts relate, and who owns their behavior? |
| Decision `rule` | Which agreed requirement must hold? |
| Decision `shape` | Why was this boundary or design chosen over the alternatives? |
| Note `spec` | What behavior should this change deliver? |

A model cites its governing Decisions. A spec and its Tasks cite the model. These references let the agent follow the reasoning without keeping independently worded copies of the same rule.

Keep a definition in the model while it is only needed there. Create a separate term Note when it needs independent lookup or is reused elsewhere. If two contexts use the same word differently, qualify their definitions instead of forcing a common meaning.

## Make the model discoverable

The skill uses the tag `domain-model` for model Notes and consistent context tags on related records. For example, the tag `operation-library` can connect a model, its terms and its Decisions. Search uses those tags and titles; it does not infer semantic relationships.

A Task that changes domain behavior names the relevant model and checks it before implementation. If an accepted rule changes, record the replacement Decision and update the model and spec to cite it. A refactor that leaves domain behavior unchanged does not need a new domain Decision.

## Use existing documentation

An existing glossary, specification or decision journal can remain the authoritative document. The notebook can index it, record its relationships and provide the context needed to resume work. Move material into separate records when doing so improves its use: a decision journal may contain several independent rulings, while a research report may have one finding worth keeping.

Preserve source links and distinguish drafts from accepted documents. When the code and a document disagree, record the discrepancy before assuming which should change. Private sources follow the notebook's [storage and sharing policy](customization.md); a shared record should explain its conclusion without requiring access to a private file.
