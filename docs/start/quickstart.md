---
title: Quickstart
description: 'Install agent-notebook, connect your agent and continue work without rebuilding context.'
---

Your agent needs shell access to run `anb`. The npm installer requires Node.js 20 or later; native binaries do not.

## Install and set up

```sh
npm install -g @supolka/agent-notebook
anb setup --agent codex
```

Run setup in the project. Choose `codex`, `claude-code` or `agents-md` for an agent that reads `AGENTS.md`; repeat `--agent` for several hosts. Setup installs the instructions and skills each selected host reads, plus its supported session-start integration. It does not create records or require a config file.

For Codex, trust the project and review the hook with `/hooks`. Start a new session so the agent can load the installed instructions. See [agent integrations](../guides/agents.md) for host details.

## Give the agent ordinary work

For example:

> Fix the export label and keep enough context to continue next time.

A small change can be one Task. The agent recalls relevant project rules and your practices, records useful progress and closes with the result and its verification. A domain model, proposal or separate investigation appears only when the work needs one.

The first record write creates `.agent-notebook/`. Shared records are Markdown files under `tasks/`, `decisions/`, `notes/` and `questions/`. Keep them in Git when they belong to the team. Commits and external actions still require the authority you gave the agent.

## Continue naturally

> Continue the export work.

> Take the next part of task.customer-exports.

> What is Grace working on?

`recall` combines current work, shared knowledge and private practices. A known session focus identifies the intended Task even when the same person has another session working elsewhere. If several Tasks could match a fresh session's request, the agent asks rather than guessing from the most recent timestamp.

For a personal work queue by default, set `scope: mine` in `.agent-notebook/config`. The work views then follow each person's assignments while project knowledge stays shared. `--team` opens the team view; `--by Grace` opens Grace's. Without that setting, work views default to the team. See [configuration](../reference/records.md#configuration).

`start --next` selects the person's ready assignments before the untaken queue. Dependencies and holds keep ineligible work out. A Task assigned to somebody else is not silently claimed. [Sessions and collaboration](../guides/session.md) explains parallel work and explicit joining.

## Remember a practice at the right scope

> For me on this project, ask before running the slow integration suite.

The agent records that practice with `--personal`, outside the repository. It is recalled automatically for you, not for colleagues.

> Across my projects, review behavior before formatting.

That practice belongs in `--global`. Shared project rules remain in the project notebook. [Personal knowledge](../guides/your-own-notebook.md) explains storage, retrieval and scope.

## The same workflow through the CLI

These explicit ids make the example reproducible. Ordinary `add` commands allocate independent random ids and return them.

```sh
anb add task "Clarify the export label" --id task.export-label --body "Rename Export to Download CSV. Preserve the action and file contents."
anb start task.export-label
anb comment task.export-label --body "The label is updated. Next: verify the download contents."
anb close task.export-label --body "The label reads Download CSV. The same action runs and the CSV fixture is unchanged."
anb archive task.export-label
anb check
```

The outcome stays on the Task. Archiving that Task does not retire related knowledge. `show` can still read its id from the archive, and `list --archive` includes finished work.

Read `anb --help` for commands and `anb <command> --help` for options. Replies use [TOON](https://toonformat.dev/); `--json` returns the same selected data as JSON. Every truncated section reports what was omitted and how to expand it.

## Customize without forking

Put project-specific instructions in `.agents/anb.md`. Setup preserves that file while updating the generated skill. Reviews, task grouping and sharing policy can change without changing the record model. See [customization](../guides/customization.md).

## Other installation options

Try the CLI without a global install:

```sh
npx -y @supolka/agent-notebook --help
```

For installed hooks, keep `anb` on PATH. [GitHub releases](https://github.com/maksimyaromin/agent-notebook/releases) provide native binaries for macOS and Linux on x64 and arm64, and Windows on x64, with checksums. Extract the appropriate binary, verify its checksum and put it on PATH.

With a Rust toolchain:

```sh
cargo install --git https://github.com/maksimyaromin/agent-notebook anb
```
