---
id: decision.the-cli-speaks-one-plain-word-per
type: decision
state: active
kind: rule
title: The CLI speaks one plain word per concept, judged by four criteria
by: Maksim Yaromin
via: claude-code
from: task.the-cli-s-vocabulary-judged-term-by-term
created: 2026-09-05
updated: 2026-09-05
---

{'lines': 2, 'head': '\nEvery word the CLI speaks — verb, flag, state value, envelope key, reply label, finding code — is judged by four criteria: the plainest word an engineer already uses for the concept, never one borrowed from another field; one word per concept across every level and the fewest words overall; a made-up synonym loses to an apt everyday word; rounded, smooth words without ragged letters, agent-notebook the model. Settled 2026-09-05 against a sweep of nineteen engineer-facing tools and the CLI-design and sound-symbolism literature. Verdicts: add is the one creation verb with the type as its first positional (add task|decision|note|question), so decide and ask fold into it; close ends Tasks and Questions alike, carrying a proof, --reason "<why>" for an ending without work or record, or --resolved-by <id> for the record a Question closed into, so answer folds into it; start also takes a Task back from review, so return folds into it; view becomes show, expunge becomes delete; Question states are open and closed with the outcome in the envelope (resolved-by or reason), routed-to becomes resolved-by, the body line a drop wrote becomes the reason field; Status says active for the working Task and keeps rules for the section that lists the Decisions of kind rule; the Debt clocks read review-stale, hold-stale and may-conflict; the finding dep-cycle reads block-cycle and broken-routing folds into missing-field and dangling-ref. Kept on the same criteria: start, submit, close, reopen, hold, unhold, block, unblock, comment, retire, ready, list, status, check, archive, restore, edit, search, graph, overview, every proof flag, tag, from, by, via, kind, priority, review-by. A new word enters only for a concept no existing word covers, and is judged the same way.\n'}
