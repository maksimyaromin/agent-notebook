---
title: Developing an idea
description: 'Keep the useful context of a proposal without creating a second tracker or a compulsory document pipeline.'
---

A feature often starts with a ticket, a page or a conversation. Use an `idea` Note when the proposal needs to survive beyond one piece of work: its motivation, open alternatives or several possible deliveries. An agreed small change can be one Task. A brief explanation may need no record at all.

The idea records the team's current understanding of the problem and intended improvement, with a link to the source. Product, support, design and engineering can contribute evidence to the same proposal. Keep what changes a later decision, not a transcript of every discussion. Permission to explore does not establish approval to deliver.

## Capture the request

Search the existing knowledge before creating another account of the proposal:

```sh
anb recall "verified operations"
```

If its context needs a home, a request to reuse verified operations can begin as:

```sh
anb add note "Reuse verified operations" --id note.verified-operations --kind idea --tag operation-library --via codex --link "doc https://example.com/issues/42" --body "Agents repeatedly prepare the same operations. Explore whether a reviewed library can reduce that work. Proposed, not approved. Open: publication rules and where operations should live."
```

The URL and explicit id make this example concrete. In ordinary work, read the actual source before summarizing it and use the id returned by the CLI for subsequent links. A title alone is not enough to infer missing requirements. The source ticket still owns its requirements, delivery commitments and discussion; the Note keeps the useful local interpretation and unresolved context.

An idea's `active` state means it is maintained. It does not mean that the proposed feature is approved. Agreement and implementation status belong in the body; the CLI does not infer either from the Note kind.

## Work out what is needed

Let the next uncertainty guide the investigation. A performance claim needs a measurement; a disputed term may need a definition; a design choice needs alternatives compared against the same outcome and constraints. Create an investigation Task when research itself has a deliverable that another session must continue. Its outcome can be evidence or a recommendation, without authorizing implementation.

Keep a brief finding or next step as a comment on the relevant record. Use a Question for an unresolved choice that affects future work, a `fact` Note for reusable evidence with sources and limits, or a `shape` Decision for an accepted design choice and its reason. Give each new record the origin that produced it, and cite supporting records where they help explain the conclusion.

The agent should bring you a concrete interpretation and recommendation, together with the question the evidence cannot settle. It should not ask you to redo the investigation, and it should not treat permission to explore as permission to implement. Existing authorization and any review gates you set determine how far it can proceed.

Create a `spec` Note only when observable behavior, exclusions and acceptance criteria need their own maintained home. A `model` Note can explain [domain relationships](domain.md). Existing specifications and models can remain canonical elsewhere; link them and record only the context missing from those sources. Neither kind is a required stage before a Task.

## Keep the connection to delivery

For one reviewable result, a Task can cite the idea directly. When several independently useful results must come together, a hub Task can connect them. Each child states its outcome, constraints and completion evidence. Keep dates, commitments and reporting in the team's tracker when that is their established home; the notebook records what the agent needs to continue the work.

The relationships have different purposes:

| Relationship | Meaning |
|---|---|
| Hub `--from` idea | Why this delivery exists |
| Child `--from` hub | Which delivery the Task belongs to |
| Hub blocked by child | What must finish before the overall result can be accepted |
| Child blocked by another child | A real prerequisite in the delivery order |
| Spec or model id cited in a body | Context the agent should read; it does not create a dependency |

The [Task guide](tasks.md#hubs-and-epics) shows the commands. These relationships are optional and serve different questions. An idea can have several deliveries without losing its identity. Keep an existing origin when adding supporting context; a related record is not automatically a prerequisite.

## Continue in another session

Ask the agent to continue the idea or delivery by name. `anb list --match <text>` finds ids, titles, tags, people and bodies, and `--tag` narrows to one subject, so consistent subject tags make the associated work easier to find; `--archive` reaches archived records.

The Task log records the established result and next action. Follow that Task's context, including an idea, hub or canonical document when present. Progress stays with the work rather than being copied into status tables across several Notes.

Archive finished Tasks when they no longer belong in the working set. Archiving a Task does not retire or archive its linked knowledge. Ideas and specs remain live while they are useful. A deferred idea can retain its reason and revisit condition without creating ready implementation work; retire it when it is no longer worth pursuing.
