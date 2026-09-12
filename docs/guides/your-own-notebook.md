---
title: Personal knowledge
description: 'Keep project-specific and cross-project practices private, and recall them without remembering their location.'
---

A project rule and a personal preference have different audiences. Shared domain knowledge belongs in the project even when one person wrote it. A preference about how an agent should work for you belongs outside the shared notebook.

## Choose the audience

| Audience | Option | Typical content |
|---|---|---|
| Project | No audience flag | Domain models, team decisions and project work |
| Personal project | `--personal` | How you want the agent to work for you on this project |
| Personal across projects | `--global` | Practices you deliberately reuse across repositories |

Personal notebooks hold Notes and Decisions. Tasks and Questions stay in a project, where assignments and unfinished work are visible in their context. Use another project notebook location if you need a private working backlog, as described under [private project memory](customization.md#keep-project-memory-private).

```sh
anb add note "Ask before the slow suite" --personal --kind guide --body "For my work on this project, ask before running the full integration suite. Focused tests can run normally."
anb add note "Review behavior first" --global --kind guide --body "Review observable behavior before formatting and naming."
```

The agent can make these choices from ordinary requests. “For me here” means personal project knowledge; “across my projects” means global knowledge. A team-wide rule needs team-wide authority. The tool does not infer that authority from who wrote the record.

## Recall without locating the file

`anb recall` combines shared project knowledge, your project-specific practices and your cross-project practices. Each memory includes its audience and a read command that selects the right notebook. Different notebooks may contain the same id without becoming the same record.

```sh
anb recall "review"
anb show note.review-practice --personal
anb show note.review-practice --global
```

The example ids in `show` stand for ids returned by creation or recall. Add the same audience flag when correcting or retiring a personal record.

A private practice does not silently override a shared rule. The agent considers their wording, authority and scope. Tags and citations do not prove a contradiction, so the CLI does not label related records as conflicting.

## Where personal knowledge lives

Cross-project records live under `.agent-notebook/` in your home directory. Project-specific practices live under its `projects/<project-key>/` directory. These paths are outside the repository and are not committed with its shared notebook.

The project key is a SHA-256 digest of the canonical local project path, or the common Git directory for a repository. Linked Git worktrees share personal practices. Independent clones do not inherit preferences merely because their directory names or remotes match. Moving a project changes its local key; move the personal notebook deliberately if its practices should follow. No remote URL, username or project title is written into a shared record to provide this mapping.

Recall reads configured sources without modifying them. A missing personal notebook is an empty source; an unreadable or malformed source is not. Storage errors remain visible.

## Shared records remain portable

A shared typed link must resolve within the shared notebook. Do not make it depend on an id that exists only in your private notebook. Summarize the shared constraint or cite an appropriate canonical source instead.

Private evidence stays private. A shared conclusion may cite a public or team-accessible source, but should not copy secrets or personal details from a private practice.

## Select another project notebook

Without an override, `anb` finds the nearest existing `.agent-notebook/` or repository root. Outside a repository it uses the working directory.

`ANB_NOTEBOOK` selects a project notebook for the shell; relative values start at the project root. `--notebook <path>` selects one for a single call; relative values start at the current working directory. `--personal` and `--global` override the environment selection and cannot be combined with each other or `--notebook`.

A custom project notebook can be committed, ignored or stored elsewhere. Selecting a different path does not turn project Tasks into cross-project personal knowledge.
