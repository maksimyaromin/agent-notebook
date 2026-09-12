---
title: Customizing the workflow
description: 'Practical recipes for private notebooks, different storage locations and your own agent workflow.'
---

The supplied workflow keeps shared project memory in the repository. Include it in code commits when the user authorizes them. You can change where the records live and how agents work with them independently. The CLI continues to check record states and relationships.

## Keep project memory private

To keep a notebook in `.tmp/xxx` without committing it, start in the repository root. Add this line to your local git exclusions file, whose path is printed by `git rev-parse --git-path info/exclude`:

```text
/.tmp/xxx/
```

Local exclusions do not change the team's `.gitignore`. They prevent untracked files from being added by ordinary git commands; they do not remove files already tracked by git.

Select the notebook before launching your agent from the same terminal:

```sh
export ANB_NOTEBOOK="$PWD/.tmp/xxx"
anb recall
```

`ANB_NOTEBOOK` names the notebook directory itself. Records go directly into `.tmp/xxx/tasks/`, `.tmp/xxx/decisions/` and the other record directories; the tool does not append `.agent-notebook/`. The first write creates the directory. Reading Recall alone does not create it.

Give the agent the corresponding working instruction:

> Use the anb skill, with project memory in `.tmp/xxx`. Keep this notebook private and out of all commits. Use the notebook selected by `ANB_NOTEBOOK` for every project read and write, including Recall and graph queries. Keep the rest of the supplied workflow.

Keep this instruction in the agent's personal instructions if you want it to apply in later sessions. Git exclusion controls which files are tracked; the instruction keeps this notebook out of otherwise authorized commits. Both belong in this setup.

### Make sure each session uses it

The export applies to this terminal and processes launched from it. Launch the agent there so its commands and session hook inherit the same notebook location. Repeat the export in a new terminal, or configure the variable in the environment that launches your agent. Writing it in a `.env` file alone does not make `anb` load it.

An agent launched separately, such as an already-running desktop application, may not inherit that environment. Set the variable through its launch configuration before using the hook. Without the override, `anb recall` reads the default project notebook alongside your personal knowledge.

This private notebook still holds project Tasks and Questions. For personal Decisions and Notes, use `--personal` within this project or `--global` across projects. See [Personal memory](your-own-notebook.md).

## Choose another location

Use the same environment variable for any notebook directory. From the repository root:

```sh
export ANB_NOTEBOOK="$PWD/project-notes"
anb recall
```

An absolute path keeps commands and hooks pointed at the same directory. A relative `ANB_NOTEBOOK` value is resolved against the project root. For one command only, use a flag:

```sh
anb --notebook .tmp/xxx recall
```

A relative `--notebook` path is resolved against that command's working directory. The flag overrides `ANB_NOTEBOOK` for this call; it does not configure later commands or hooks. [Notebook location](your-own-notebook.md#select-another-project-notebook) gives the full precedence rules.

Changing the location selects a different notebook. It does not move or import existing records. To relocate an existing notebook, stop agents using it, move the complete notebook directory to the new location, and update their environment. Run `anb check` against the destination before resuming.

## Add project workflow rules

Put team-specific workflow rules in `.agents/anb.md` at the project root. The supplied `anb` skill reads this optional file before working with the notebook, whether installed for Claude Code, Codex or the agents.md convention. Keep it brief: name the project exceptions, external sources of truth and review requirements. Command syntax stays in the generated references.

For example:

```md
# Project workflow

Asana owns delivery dates and issue status. Link the Asana task from notebook work; record only local decisions, blockers and the next action here.

Submit verified work for human review. Close and archive it after acceptance.
```

Setup never creates, overwrites or removes `.agents/anb.md`, so the same project policy survives skill updates for every agent. Commit the file when the rules apply to the team. Keep your own writing preferences and working habits in [your own notebook](your-own-notebook.md); they do not belong in the shared workflow.

## Replace a skill

Setup installs the workflow where each named agent looks for skills: `.claude/skills/anb/` for Claude Code, `.agents/skills/anb/` for Codex and the agents.md convention. Edit the copy your agent reads; if you wire both, apply the same changes to both copies.

1. Open `SKILL.md` and remove its `managed-by: anb` metadata entry. Keep the skill's name and description.
2. Change the working instructions. Leave command syntax to the generated references unless you need to change those too.
3. Run `anb setup --agent <name>` again. It should report the edited file as `yours, left alone`.

Ownership is per file. Remove the marker from each reference you customize too. Unchanged managed references can still receive updates. For a private skill, exclude its files locally as in the notebook recipe above; commit them when the workflow should be shared with the team.

This protection applies to `anb setup`, including `setup --remove`. `anb skill <dir>` is a generator that writes its output to the named directory, so use a separate directory when comparing a new generated skill with your custom version.

You can apply the same approach to `anb-atlas` to change how the agent presents or reviews work. You can also replace the supplied skills entirely and use the [command reference](../reference/commands.md) to define your method.

## Require review before closing

Add a review rule to `.agents/anb.md`:

> When implementation and verification are complete, log the result and submit the Task for human review. Leave it in review until I accept it. If I request changes, start the Task again. After acceptance, close with a short outcome and archive it.

The commands already support this workflow:

```sh
anb submit <task-id>
anb start <task-id>
anb close <task-id> --body "Accepted after review. The focused tests pass."
anb archive <task-id>
```

These are alternatives at different points in review: `submit` requests acceptance, `start` resumes changes, and `close` records the accepted result. The CLI checks transitions; your instructions determine who accepts the work.

## Change how you organize work

You can keep a flat queue, use the supplied epic pattern, or run several agents on separate Tasks. Add project-specific coordination rules to `.agents/anb.md` when needed:

> Begin with Recall. Resume this session's remembered Task or select ready work with `start --next`. Give parallel conversations separate session ids. Log a handoff before another agent joins the same work; use `edit --taken-by` when handing it to another person.

The CLI allows multiple active Tasks. `start` records who holds a Task as `taken-by` and refuses a Task someone else holds; each agent's identity comes from `ANB_BY` or the git `user.name` of its checkout. Session claims also prevent two local conversations from silently choosing the same work, unless they explicitly join it. Neither session claims nor the write lock prevent two agents from editing the same source file, and neither is a distributed lease across clones. Your coordination rules need to cover that.

One person can plan for the others: `anb add task "<title>" --taken-by <name>` writes a Task and hands it over in one command. A Task without a holder remains in the pool until someone starts it. With `scope: mine` in `.agent-notebook/config`, routine views select the Tasks the caller holds, the records the caller wrote and the records waiting on the caller. Explicit knowledge queries using `--type note`, `--type decision` or `--kind` include team authors by default: project knowledge applies whoever recorded it. An explicit `--mine` or `--by <name>` still narrows those queries. Use `--team` to inspect the team or `anb ready --untaken` to choose work from the pool.

Tags can express your own categories. Automatic epic progress still follows the [hub relationships](tasks.md#hubs-and-epics): a hub depends on work created from it. A different tag alone does not change that calculation. You can change the convention agents follow while retaining those relationships when you want the built-in epic summary.
