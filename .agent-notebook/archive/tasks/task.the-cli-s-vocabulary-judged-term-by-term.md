---
id: task.the-cli-s-vocabulary-judged-term-by-term
type: task
state: closed
title: The CLI's vocabulary, judged term by term
by: Maksim Yaromin
via: claude-code
from: task.improvements
tags: cli
link: note note.report-the-cli-s-vocabulary-judged-term
priority: 1
created: 2026-09-05
updated: 2026-09-05
closed: 2026-09-05
---

Owner's request 2026-09-05, restated after two refused words (drop, withdraw): this is serious analytical work, not a pass over the help text. Read every word the CLI speaks at every level — verbs, flags, state values, envelope keys, kinds, error and finding codes, the words a reply and a refusal print, the help texts, the glossary — and judge each on four stated criteria: (1) an engineering product for engineers: the plainest word an engineer already uses for this, never a term borrowed from another field; (2) unification: one word for one concept across every level, the fewest distinct words in the whole vocabulary; (3) no over-complication: a made-up synonym is worse than an apt everyday word; (4) the owner's ear and eye: rounded, smooth words without ragged letters — agent-notebook is the model, withdraw the counterexample. Method: the full inventory first; then a systematic sweep of what engineer-facing trackers and CLIs call the same concepts (git, gh, jira, linear, beads, tasks-axi, taskwarrior and their kin), findings classified tried-versus-theory, run on Sonnet 5 as the research method requires; then a verdict per term with the reason and the rejected alternatives. Deliverables: the analysis report under .tmp/docs; one Decision recording every verdict, kept and changed alike; CONTEXT.md rewritten to the settled vocabulary; the renames applied across code, tests, help and error texts, the old words swept by search. Words that live in the file format — state values, envelope keys, type and kind words — bind every existing notebook: a rename carries its migration in the same change, or the word stays and the Decision says why. Acceptance: check green on this repository's own notebook; every command in the help runs as printed; the Task's reasoned ending (task.close-drop-ends-a-task-without-work) takes its word from here; the generated skill and the docs inherit the settled words, so this closes before the skill generator starts.
- 2026-09-05 Maksim Yaromin: Done 2026-09-05: report at .tmp/docs/report-cli-vocabulary.md, sweep at research/18, inventory at research/17, decision.the-cli-speaks-one-plain-word-per. Smoke check (Sonnet 5): 9 findings — one must-fix (the resolve replay's reply named the resolver the call carried, not the one the record holds; fixed and guarded by a test), two stale module docs, five stale words in comments and test names, all taken. Gate green; this notebook migrated; check green.
