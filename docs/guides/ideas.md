---
title: Developing an idea
description: 'Bring a request into the notebook, investigate it and keep its connection to delivery.'
---

A feature often starts with a ticket, a page or a conversation. That description gives a developer something to investigate. The supplied skill starts a Note of kind `idea` for this work, or resumes an existing one about the same subject.

The idea records the problem and intended improvement in the developer's own understanding, with a link to the source. It can accumulate questions, alternatives and evidence before implementation is agreed. Keeping it separate from a Task means the proposal can remain useful even if it produces several deliveries or no code at all.

## Capture the request

For example, a request to reuse verified operations can begin as:

```sh
anb add note "Reuse verified operations" --kind idea --tag operation-library --via codex --link "doc https://example.com/issues/42" --body "Agents repeatedly prepare the same operations. Explore whether a reviewed library can reduce that work. Direction: proposed. Open: publication rules and where operations should live."
```

The URL is an example source. An agent reads the actual source before summarizing it and uses the returned record id for subsequent links. A title alone is not enough to infer missing requirements.

An idea's `active` state means it is maintained. It does not mean that the proposed feature is approved. Agreement and implementation status belong in the body; the CLI does not infer either from the Note kind.

## Work out what is needed

The skill uses uncertainties to guide the investigation. For sustained research or design work, the agent creates an investigation Task from the idea. Status can then resume that work without treating the feature itself as approved for implementation. A disputed term may need a definition and a domain model. A performance claim may need a measurement. A choice between implementations needs a comparison against the same outcome and constraints.

Questions record what remains unresolved, using the idea or investigation Task as their origin. Research findings become `fact` Notes with sources and limits. A design choice becomes a `shape` Decision once it is settled. The idea cites these records so another session can follow the reasoning.

The agent should bring you a concrete interpretation and recommendation, together with the question the evidence cannot settle. It should not ask you to redo the investigation, and it should not treat permission to explore as permission to implement. Existing authorization and any review gates you set determine how far it can proceed.

A small change may need only the idea and a Task. For a larger change, a `spec` Note defines observable behavior, exclusions and acceptance criteria. A `model` Note explains the [domain relationships](domain.md). These documents are created when they help the work; the workflow does not require a fixed document set.

## Keep the connection to delivery

When the work is ready for delivery planning, create a hub Task from the idea and child Tasks from that hub. The hub cites the relevant spec and Decisions. Each child describes a reviewable result, the behavior to preserve, any remaining research and how completion will be checked.

The relationships have different purposes:

| Relationship | Meaning |
|---|---|
| Hub `--from` idea | Why this delivery exists |
| Child `--from` hub | Which delivery the Task belongs to |
| Hub blocked by child | What must finish before the overall result can be accepted |
| Child blocked by another child | A real prerequisite in the delivery order |
| Spec or model id cited in a body | Context the agent should read; it does not create a dependency |

The [Task guide](tasks.md#hubs-and-epics) shows the commands. An idea can have several delivery hubs without losing its identity. A new finding gets the origin that actually produced it; the agent cites other relevant records in its body instead of repeatedly reassigning `from`.

## Continue in another session

Ask the agent to continue the idea or delivery by name. Search finds ids, titles and tags, including archived records; it does not search the body. Consistent subject tags make the associated work easier to find.

The Task log records the established result and next action. The hub points to the idea and governing documents. Progress stays in Tasks rather than being copied into status tables across the idea, spec and plan.

Finished Tasks are archived with their reports. Ideas and specs remain live while they are useful. A deferred idea can retain its reason and revisit condition without creating ready implementation work. Retire it when it is no longer worth pursuing.
