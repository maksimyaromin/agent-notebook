---
id: task.spike-agent-interaction
type: task
state: closed
title: Spike: agent interaction
by: Maksim Yaromin
via: claude-code
tags: spike
link: note note.report-spike-agent-interaction
blocked-by: task.spike-record-model
created: 2026-08-24
updated: 2026-08-25
closed: 2026-08-25
---

Literature sweep (token-cost measurements of output encodings in the literature). Measure TOON vs JSON vs plain on fixtures, in tokens; hook designs for Claude Code, Codex, OpenCode, Pi; skill generation scheme from CLI help. Close: spec update.

## Progress log

- 2026-08-25: own measurement done: TOON vs JSON vs plain on anb fixtures, o200k_base (official TOON pkg + gpt-tokenizer, cl100k cross-check <2% delta). plain-table ties TOON on uniform rows (-24.3% vs JSON both), beats it by 36pp on non-uniform list (TOON +19.4% WORSE than compact JSON there), designed plain Status -21.5% vs JSON and 19pp better than TOON (212 tok / 60-task notebook). TOON finally rejected on S1 literature + own numbers; shape-matched plain confirmed; --json stays compact JSON; omission/bounds remain first-order lever.
- 2026-08-25: sweep 14 (tool docs/skills) landed: 16 verified cards. Headlines: structured error payloads +36.7-40pp task completion vs prose (2606.05037), gains compound over retries (2509.18847); worked examples >> abstract schemas (2308.00675; LangChain A/B 16->52%); ~600-word doc ceiling hypothesis; one rewrite pass ~= hand-tuned skill descriptions (2606.30775); SKILL.md contract consumed by all four target agents — one generated artifact portable; GAP: no OSS tool CI-checks a generated skill against live CLI behavior — anb first to close it.
- 2026-08-25: Hook mechanics surveyed. Feasibility: Claude Code clean (project .claude/settings.json, SessionStart, additionalContext); Codex possible (.codex/hooks.json mirrors CC schema, one-time /hooks trust, ~2500-token additionalContext cap); OpenCode unreliable (no session-start event; chat.system.transform broken per maintainer issue #17100 -> AGENTS.md only reliable path); Pi needs a shipped TS extension (session_start + sendMessage) but auto-loads AGENTS.md. Premise corrections: Backlog.md DOES ship MCP. Recommended anb setup: AGENTS.md snippet as floor; project-scoped hooks for CC/Codex by default; payload cap 1500-2000 tok; marker-based idempotent diff/remove; OpenCode plugin opt-in; fail soft.
- 2026-08-25: sweep 12 (context-injection) landed: 19 verified cards. Decision-critical: AGENTS.md prose null-to-negative +20% cost (12#1-3) vs uninferable-state dashboard 2.2x task success (12#15); unconditional session-start injection is a NAMED BEATEN baseline (12#16-17: selective wins +8.3pp) -> design answer: deterministic injection gate; in-session decay 5.6%/step (12#2) -> micro-Status in mutation replies + compact-matcher re-injection; conflict resolution ~48% ceiling (12#7) -> Status descriptive never imperative; injection safety: titles-only + framing line + documented residual risk (static delimiters bypassable 12#12, randomized+datamark <2% ASR 12#11 deliberately not adopted for local trust boundary).
- 2026-08-25: design done: the interaction spec (8 sections: encoding call closed with own numbers; structured errors error[code]+try:; gated Status w/ degrade order + budget 1500; safety posture; setup per-agent matrix project-scoped+markers+fail-soft; AGENTS.md snippet text; skill single-source + CI byte-diff gate + 600-word instrumented hypothesis; out-of-S3).
