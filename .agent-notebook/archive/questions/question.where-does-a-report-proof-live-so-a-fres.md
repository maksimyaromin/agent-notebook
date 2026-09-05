---
id: question.where-does-a-report-proof-live-so-a-fres
type: question
state: closed
title: Where does a report proof live so a fresh clone can reach it?
by: Maksim Yaromin
via: claude-code
from: task.milestone-self-host-switch
resolved-by: decision.close-proofs-are-equals-default-note
created: 2026-08-29
updated: 2026-08-29
---

Closed records committed in the notebook carry report proofs pointing into git-ignored .tmp (the self-host migration imported ten of them); a fresh clone reads a proof path it cannot open, and close accepts any string unverified. Candidate answers: prefer --sha/--pr once commits exist (git-reachable, self-contained), give the notebook its own committed artifacts area for reports, or rule that report proofs are machine-local by convention and say so in the docs.
