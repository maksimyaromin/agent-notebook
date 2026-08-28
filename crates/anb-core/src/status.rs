//! Status: the budgeted session-start dashboard the Core assembles from
//! records.
//!
//! Only synthesized, uninferable state enters — never prose, never
//! instructions: counts,
//! the in-flight Task with its last log line, review Tasks waiting on a
//! human, standing rules, the ready top rows, Debt. The gate keeps a quiet
//! notebook to one line. Under a Budget the sections degrade in fixed order
//! — ready rows first, then Debt to a count, then rules to a count, then
//! the log and review lines — and the in-flight line is never dropped.
//! Every full dashboard ends with the budget line; when something was cut,
//! it names the cut and carries the command that restores it.

use crate::debt::DebtSignal;
use crate::encode::json_quoted;
use crate::notebook::ReadyTask;
use crate::tokens::estimate_tokens;
use std::fmt::Write as _;

/// How many ready rows a full dashboard shows; more exist behind the
/// truncation hint.
const READY_ROWS: usize = 5;

/// The token ceiling a Status must fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    Tokens(u32),
    Unbounded,
}

impl Budget {
    pub const DEFAULT_TOKENS: u32 = 1500;

    /// The one spelling of "no ceiling", shared by the config key and the
    /// CLI flag: a ceiling of `0` disables the Budget.
    #[must_use]
    pub fn from_ceiling(ceiling: u32) -> Budget {
        if ceiling == 0 {
            Budget::Unbounded
        } else {
            Budget::Tokens(ceiling)
        }
    }

    fn holds(self, spent: u32) -> bool {
        match self {
            Budget::Tokens(ceiling) => spent <= ceiling,
            Budget::Unbounded => true,
        }
    }
}

/// Live records per type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub tasks: usize,
    pub decisions: usize,
    pub notes: usize,
    pub questions: usize,
}

/// An active Task on the dashboard: the `in-flight:` line, plus the last
/// log line for the one most recently touched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTask {
    pub id: String,
    pub title: String,
    pub log: Option<String>,
}

/// A standing rule: a live Decision of kind `rule`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRule {
    pub id: String,
    pub title: String,
}

/// The assembled dashboard: the budgeted plain text and the model it
/// renders. `ready` holds the whole queue; the text bounds it.
#[derive(Debug, PartialEq, Eq)]
pub struct Status {
    pub text: String,
    pub quiet: bool,
    pub counts: Counts,
    pub in_flight: Vec<ActiveTask>,
    pub review: Vec<String>,
    pub rules: Vec<StatusRule>,
    pub ready: Vec<ReadyTask>,
    pub debt: Vec<DebtSignal>,
    /// The budget line's number: the body's and the line's own estimates
    /// summed, so it never falls under the whole text's estimate and may
    /// exceed it by the one rounding token.
    pub spent: u32,
}

pub(crate) struct StatusInputs {
    pub counts: Counts,
    pub in_flight: Vec<ActiveTask>,
    pub review: Vec<String>,
    pub rules: Vec<StatusRule>,
    pub ready: Vec<ReadyTask>,
    pub debt: Vec<DebtSignal>,
    pub today_day: i64,
}

impl StatusInputs {
    /// The gate: signal is work in motion, work waiting on a human,
    /// dispatchable work, or decay. Review counts deliberately: a Task
    /// parked at acceptance is not a quiet notebook.
    fn has_signal(&self) -> bool {
        !self.in_flight.is_empty()
            || !self.review.is_empty()
            || !self.ready.is_empty()
            || !self.debt.is_empty()
    }

    fn has_log(&self) -> bool {
        self.in_flight
            .first()
            .is_some_and(|task| task.log.is_some())
    }
}

/// How far the degradation has gone past trimming ready rows; each stage
/// implies every earlier one, so an out-of-order state is unrepresentable.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Collapse {
    #[default]
    Nothing,
    Debt,
    Rules,
    Log,
    Floor,
}

/// The degradation ladder: each rung buys tokens by collapsing one section,
/// in the fixed order the module doc states; the floor keeps counts, the
/// in-flight lines, and the budget line.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Ladder {
    ready_trimmed: usize,
    collapsed: Collapse,
}

impl Ladder {
    /// Tighten one rung; `false` when already at the floor.
    fn tighten(&mut self, ready_shown: usize) -> bool {
        if self.ready_trimmed < ready_shown {
            self.ready_trimmed += 1;
            return true;
        }
        let next = match self.collapsed {
            Collapse::Nothing => Collapse::Debt,
            Collapse::Debt => Collapse::Rules,
            Collapse::Rules => Collapse::Log,
            Collapse::Log => Collapse::Floor,
            Collapse::Floor => return false,
        };
        self.collapsed = next;
        true
    }

    fn reached(self, stage: Collapse) -> bool {
        self.collapsed >= stage
    }

    /// What the budget line reports as cut, in ladder order.
    fn cut_note(self, ready_shown: usize, inputs: &StatusInputs) -> Option<String> {
        if self == Ladder::default() {
            return None;
        }
        if self.reached(Collapse::Floor) {
            return Some("all but in-flight".to_owned());
        }
        let mut cuts = Vec::new();
        if self.ready_trimmed > 0 {
            cuts.push(format!(
                "ready {ready_shown}\u{2192}{}",
                ready_shown - self.ready_trimmed
            ));
        }
        if self.reached(Collapse::Debt) && !inputs.debt.is_empty() {
            cuts.push("debt\u{2192}count".to_owned());
        }
        if self.reached(Collapse::Rules) && !inputs.rules.is_empty() {
            cuts.push("rules\u{2192}count".to_owned());
        }
        if self.reached(Collapse::Log) {
            if inputs.has_log() {
                cuts.push("log".to_owned());
            }
            if !inputs.review.is_empty() {
                cuts.push("review\u{2192}count".to_owned());
            }
        }
        (!cuts.is_empty()).then(|| cuts.join(", "))
    }
}

/// Assemble the dashboard under `budget`: render, measure, and walk the
/// ladder until the text fits or the floor is reached — the floor ships
/// even over budget, reported honestly on the budget line.
pub(crate) fn assemble(inputs: StatusInputs, budget: Budget) -> Status {
    if !inputs.has_signal() {
        let text = quiet_line(&inputs.counts);
        let spent = estimate_tokens(&text);
        return status_from(inputs, text, spent, true);
    }

    let ready_shown = inputs.ready.len().min(READY_ROWS);
    let mut ladder = Ladder::default();
    loop {
        let (text, spent) = render(&inputs, ladder, ready_shown, budget);
        if budget.holds(spent) || !ladder.tighten(ready_shown) {
            return status_from(inputs, text, spent, false);
        }
    }
}

fn status_from(inputs: StatusInputs, text: String, spent: u32, quiet: bool) -> Status {
    Status {
        text,
        quiet,
        counts: inputs.counts,
        in_flight: inputs.in_flight,
        review: inputs.review,
        rules: inputs.rules,
        ready: inputs.ready,
        debt: inputs.debt,
        spent,
    }
}

fn quiet_line(counts: &Counts) -> String {
    format!(
        "ok: notebook quiet — {}. anb --help when needed.\n",
        counts_phrase(counts)
    )
}

fn counts_phrase(counts: &Counts) -> String {
    format!(
        "{} tasks, {} decisions, {} notes, {} questions",
        counts.tasks, counts.decisions, counts.notes, counts.questions
    )
}

/// One full render under a ladder state, budget line included. The spent
/// number sits inside the line it reports, so the measure re-runs with the
/// number filled in, at most three passes; the last measure wins, so a
/// digit-count flip on the final pass can leave the printed number one off.
fn render(
    inputs: &StatusInputs,
    ladder: Ladder,
    ready_shown: usize,
    budget: Budget,
) -> (String, u32) {
    let body = render_body(inputs, ladder, ready_shown);
    let cut = ladder.cut_note(ready_shown, inputs);
    let mut spent = 0;
    for _ in 0..3 {
        let line = budget_line(spent, budget, cut.as_deref());
        let measured = estimate_tokens(&body) + estimate_tokens(&line);
        if measured == spent {
            break;
        }
        spent = measured;
    }
    let text = format!("{body}{}", budget_line(spent, budget, cut.as_deref()));
    (text, spent)
}

fn budget_line(spent: u32, budget: Budget, cut: Option<&str>) -> String {
    match (budget, cut) {
        (Budget::Unbounded, _) => format!("budget: ~{spent} tokens (no ceiling)\n"),
        (Budget::Tokens(ceiling), None) => format!("budget: ~{spent}/{ceiling} tokens\n"),
        (Budget::Tokens(ceiling), Some(cut)) => {
            format!("budget: ~{spent}/{ceiling} tokens; cut: {cut} — anb status --budget 0\n")
        }
    }
}

fn render_body(inputs: &StatusInputs, ladder: Ladder, ready_shown: usize) -> String {
    let mut out = format!("ok: notebook — {}\n", counts_phrase(&inputs.counts));
    for (position, task) in inputs.in_flight.iter().enumerate() {
        let _ = writeln!(out, "in-flight: {} {}", task.id, json_quoted(&task.title));
        if position == 0
            && !ladder.reached(Collapse::Log)
            && let Some(log) = &task.log
        {
            let _ = writeln!(out, "log: {log}");
        }
    }
    if ladder.reached(Collapse::Floor) {
        return out;
    }
    render_review(&mut out, &inputs.review, ladder);
    render_rules(&mut out, &inputs.rules, ladder);
    render_ready(
        &mut out,
        &inputs.ready,
        ready_shown - ladder.ready_trimmed,
        inputs.today_day,
    );
    render_debt(&mut out, &inputs.debt, ladder);
    out
}

fn render_review(out: &mut String, review: &[String], ladder: Ladder) {
    if review.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Log) {
        let _ = writeln!(out, "review: {}", review.len());
    } else {
        let _ = writeln!(
            out,
            "review[{}]: {} — waiting on a human",
            review.len(),
            review.join(", ")
        );
    }
}

fn render_rules(out: &mut String, rules: &[StatusRule], ladder: Ladder) {
    if rules.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Rules) {
        let _ = writeln!(out, "rules: {}", rules.len());
        return;
    }
    let _ = writeln!(out, "rules[{}]:", rules.len());
    for rule in rules {
        let _ = writeln!(out, "  {}: {}", rule.id, rule.title);
    }
}

fn render_ready(out: &mut String, ready: &[ReadyTask], shown: usize, today_day: i64) {
    if ready.is_empty() {
        return;
    }
    if shown == 0 {
        let _ = writeln!(out, "ready: {} — anb ready", ready.len());
        return;
    }
    out.push_str(&crate::encode::ready_table(ready, shown, today_day));
    if ready.len() > shown {
        let _ = writeln!(out, "  … {} more: anb ready", ready.len() - shown);
    }
}

/// How many lines each mention-borne class may spend before its hint; the
/// hint is one line, shorter than what it truncates. The clock classes list
/// in full — the notebook itself bounds them.
const MENTION_CLASS_ROWS: usize = 5;

fn render_debt(out: &mut String, debt: &[DebtSignal], ladder: Ladder) {
    if debt.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Debt) {
        let _ = writeln!(out, "debt: {}", debt.len());
        return;
    }
    let _ = writeln!(out, "debt[{}]:", debt.len());
    for (bounded_class, plural) in [
        (None, ""),
        (Some("dangling-mention"), "dangling mentions"),
        (Some("undeclared-pair"), "undeclared pairs"),
    ] {
        let mut shown = 0;
        let mut hidden = 0;
        for signal in debt {
            let in_class = match bounded_class {
                None => !matches!(signal.code(), "dangling-mention" | "undeclared-pair"),
                Some(code) => signal.code() == code,
            };
            if !in_class {
                continue;
            }
            if bounded_class.is_some() && shown == MENTION_CLASS_ROWS {
                hidden += 1;
                continue;
            }
            let _ = writeln!(out, "  {}", signal.line());
            shown += 1;
        }
        if hidden > 0 {
            let _ = writeln!(out, "  \u{2026} {hidden} more {plural}");
        }
    }
}
