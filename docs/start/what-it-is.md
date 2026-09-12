---
title: What it is
description: 'How project knowledge, personal practices and unfinished work become useful memory for the next session.'
---

A new agent can read the code. It cannot read yesterday's conversation, know which interpretation the team rejected or guess which unfinished change you mean by “continue.” Those gaps cost attention every time someone starts again.

agent-notebook keeps the context worth carrying across that boundary. It gives the project a shared account of what things mean, the person a view of their work and the conversation a place to resume. Its purpose is continuity of understanding, not completeness of the record.

## Keep the conclusion, not the conversation

A useful memory changes a later decision. “The request timed out” records an attempt. “This endpoint is unreliable” makes a broader claim that the attempt alone cannot support. The notebook preserves the attempt on its Task; a maintained Note needs evidence for the conclusion it offers.

This distinction separates quick capture from consolidation. Capture records what happened, what remains uncertain and the next useful action. Consolidation reads that evidence alongside the existing explanation and updates what the project can rely on. It preserves constraints that still hold and names a replacement when the ruling changes. It is not a shorter transcript of the latest conversation.

The record types follow those different lifetimes. A Task ends when its result is verified. A Question ends when its uncertainty is settled. A Note remains useful while its explanation is maintained. A Decision remains applicable until it is replaced or retired. Finishing the Task does not erase the knowledge learned from it. A small change needs no proposal, specification or separate report merely to complete this cycle.

## Remember enough to act

Stored memory and active context have different jobs. The files preserve detail; Recall supplies a bounded starting point. The agent follows the user's subject, reads nearby knowledge and opens the sources needed for the decision. Counts and omissions disclose what the initial reply left out. A short excerpt is an invitation to read further, not evidence that nothing else matters.

Retrieval does not change the records. Reading a rule should not increase its authority, alter its date or make competing explanations disappear. Related statements may complement each other. Only their meaning, evidence and scope can establish a contradiction.

Review begins when new evidence challenges a claim, a planned review becomes due or someone asks to reconsider it. The agent compares the relevant material and makes a deliberate correction. Age can prompt attention; it cannot justify automatic loss of knowledge.

### What biological memory contributes

Research on complementary learning systems distinguishes rapid learning of particular experiences from the integration of structured knowledge. That distinction informs the separation between work logs and maintained explanations. It does not establish a biological equivalence between files and neural systems. [Kumaran, Hassabis and McClelland, 2016](https://pubmed.ncbi.nlm.nih.gov/27315762/).

Experiments on schemas show that established knowledge can affect how new information is learned. The engineering consequence here is to read the existing model before creating another account of the same subject. A supported observation can refine that model; a single example does not establish a universal rule. [Tse and colleagues, 2007](https://pubmed.ncbi.nlm.nih.gov/17412951/).

These are design inferences, not borrowed operating parameters. Human working-memory estimates do not establish a four-record limit for an LLM. The notebook uses an explicit output budget, not a simulated attention span. It has no confidence coefficients, retrieval penalties, random forgetting or mandatory sleep cycle. Each would need independent evidence that it improves this software's behavior. [Cowan, 2001](https://pubmed.ncbi.nlm.nih.gov/11515286/).

## Share meaning without mixing responsibilities

Product, support and engineering encounter different parts of the same system. Their agents need to agree on what a Publication is, which boundary protects a tenant and why an export may be refused. A definition belongs to the project whoever wrote it. Where contexts use the same word differently, the model explains the difference instead of forcing a misleading universal definition.

A PM can bring an idea without designing its implementation. The idea preserves the problem, intended result and unresolved choices. The team can investigate it, record a ruling and deliver it through related Tasks. The proposal remains distinguishable from an approved commitment.

Authorship, responsibility and attention stay separate. Creating a Task records work that could be done; starting it records who takes it. A Task representing a larger result can have independently assigned parts. Its owner is accountable for the overall result, not automatically the executor of every part. With a session identity supplied by the host or caller, each conversation remembers the Task it is working on, so one person can have several sessions without one session silently redirecting another.

“What is on my list?” selects personal work. “What is Alex doing?” changes the view, not the assignment. “Work on the export redesign” requires finding the intended result and inspecting its parts before taking responsibility. Existing assignments remain visible during that inspection. The agent asks before taking overlapping work, not after it has reassigned someone else's Tasks.

Composition, prerequisites and context answer different questions. A migration needed by the export redesign may be another team's work. It can block a delivery without becoming part of that delivery. A source document explains the work without becoming work to assign. [Tasks and the shape of work](../guides/tasks.md) defines these relationships and their commands.

Personal practices have a different audience from project facts. “Ask me before running the slow suite here” belongs to that person's project memory. A practice deliberately used across projects belongs to their global memory. Both stay outside the repository and appear with their audience labelled. A personal preference neither changes a shared definition nor silently outranks a team requirement.

## Leave useful files behind

The next reader may have no CLI, hook or skill. A record must therefore explain itself: a meaningful title, a conclusion with its scope, the reason or evidence behind it and links to the sources a reader can inspect. An id keeps references stable; it does not replace the name and meaning of the thing being referenced.

The YAML envelope exposes state, attribution and relationships. The Markdown body carries the explanation. File search that includes the notebook can find the project's vocabulary, and a reader can distinguish a proposal from a ruling or an old ruling from its replacement. Setup leaves a discovery paragraph in the project's native agent instructions, including how to read the files when the command is unavailable. A project can also link the notebook from its ordinary documentation.

This makes the memory portable, not omniscient. An agent still has to discover and read the relevant files. A private source must remain private, and a shared conclusion must not depend on a teammate's local preferences. The notebook keeps a useful interpretation of a ticket or document; the original system retains its commitments and published explanation.

## Put each guarantee in the right place

The CLI owns facts it can check: record structure, lifecycle transitions, references, dependency cycles and safe file changes. It derives views from the records rather than persisting another dashboard or search database. JSON and TOON encode the same selected reply. Failures identify what happened and the next useful read or correction.

The skill owns judgment: what deserves memory, which result the user means, what evidence supports a claim and when another person's agreement is needed. Project workflow rules belong in `.agents/anb.md`; they do not require a fork of the installed skill. This boundary follows the software-design principle of hiding machinery behind a small, meaningful interface. A field or command earns its place by expressing a distinct fact, not by anticipating every phrase a user might say.

Files are shared through Git. Local locks protect cooperating writers; session claims coordinate conversations using the same notebook. They do not create a distributed lease across disconnected clones. A view of a colleague's work reports the recorded state available in the checkout, not a live view of their computer.

The result is a memory the team can inspect and correct, and a routine the agent can resume. [Start with one piece of work](quickstart.md). The [record reference](../reference/records.md) explains the data contract; the [working method](../guides/session.md) explains a session.
