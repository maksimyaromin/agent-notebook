---
id: note.report-check-atlas-guidance-against-the
type: note
state: retired
title: Report: Check atlas guidance against the author method and graph contract
by: Maksim Yaromin
from: task.atlas-alignment
created: 2026-09-06
updated: 2026-09-06
---

# Align atlas review with the author method

Reviewed the atlas skill, drawing reference, intent loop and public guide against graph serialization, graph edge construction and actual CLI commands. Preserved the layout and interaction design. The atlas now uses the main method before applying review comments, distinguishes record type from Note/Decision kind, preserves notebook selection and the captured record set, and handles typed lifecycles, active-task handoff, partial batches and replay. The graph reference describes directed-pair edge precedence and the public guide correctly requires --all to remove envelope/body limits.

Removed description quotes from the generated main skill and references at the user's request. Rephrased descriptions containing colon-space sequences so all descriptions remain plain YAML scalars. The atlas main description originally failed YAML parsing; all seven skill/reference frontmatters now parse successfully.

## Evidence

Before editing, the old duplicate-close recipe failed for an idea Note with wrong-type. A long Note demonstrated that graph JSON always carries all selected nodes and edges but truncates body text without --all. The initial revised duplicate recipe also failed because comment accepts Task logs, not Notes. Corrected it to preserve the body and append the duplicate reason with edit before retirement and archival; the pending step then passed.

An independent scratch CLI exercise applied a review batch containing a Task switch, a duplicate idea Note and archive-everything-closed. The old Task received a via-labelled handoff and hold, the selected Task became the sole unheld active Task, the duplicate was retired and archived with original text and its survivor citation retained, and only the closed Task captured by the page was archived. A closed Task added after capture remained byte-for-byte unchanged. Both partial and completed-batch replay left notebook hashes unchanged. Check retained the expected out-of-slice unarchived-record warning; the exercise did not broaden the user's batch merely to remove it. Holding is orthogonal to Task state, so one in-flight Task does not mean only one raw active state.

Validation covers command application and serialization, not a generated page. No browser rendering or interaction claim is made because this change produced no HTML. The presentation guidance was not redesigned. Workspace checks and docs checks passed; generated files match the binary and git diff --check is clean. No commit, push or external-system change was performed.
