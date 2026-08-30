//! Status: the budgeted session-start dashboard the Core assembles from
//! records.
//!
//! Only synthesized, uninferable state enters — never prose, never
//! instructions: counts,
//! the in-flight Task with its last log line, review Tasks waiting on a
//! human, standing rules, the ready top rows, where each epic stands, Debt.
//! The gate keeps a quiet notebook to one line. Under a Budget the sections
//! degrade in fixed order — ready rows first, then epics to a count, then
//! Debt to a count, then rules to a count, then the log and review lines —
//! and the in-flight line is never dropped: at the floor all but the first
//! collapse to a count, so no notebook, however much it holds in flight,
//! can make a Status grow without bound.
//! Every full dashboard ends with the budget line; when something was cut,
//! it names the cut and carries the command that restores it.

use crate::debt::DebtSignal;
use crate::encode::{self, json_quoted};
use crate::notebook::{Epic, ReadyTask};
use crate::tokens::estimate_tokens;
use std::fmt::Write as _;

/// How many rows any one section of the dashboard shows before it names the
/// rest as a count. Status is a fixed opening, not a report. A section that
/// grew with the notebook would spend a whole session's opening on itself,
/// and it would do so at the worst moment: the Debt list is longest exactly
/// when the least is tended.
pub const SECTION_ROWS: usize = 5;

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
/// renders. Each list holds all of what it counts; the renderings bound it.
#[derive(Debug, PartialEq, Eq)]
pub struct Status {
    pub text: String,
    pub quiet: bool,
    pub counts: Counts,
    pub in_flight: Vec<ActiveTask>,
    pub review: Vec<String>,
    pub rules: Vec<StatusRule>,
    pub ready: Vec<ReadyTask>,
    pub epics: Vec<Epic>,
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
    pub epics: Vec<Epic>,
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
    Epics,
    Debt,
    Rules,
    Log,
    Floor,
}

/// The degradation ladder: each rung buys tokens by collapsing one section,
/// in the fixed order the module doc states; the floor keeps counts, the
/// first in-flight line, and the budget line.
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
            Collapse::Nothing => Collapse::Epics,
            Collapse::Epics => Collapse::Debt,
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
            return Some("all but the first in-flight".to_owned());
        }
        let mut cuts = Vec::new();
        if self.ready_trimmed > 0 {
            cuts.push(format!(
                "ready {ready_shown}\u{2192}{}",
                ready_shown - self.ready_trimmed
            ));
        }
        if self.reached(Collapse::Epics) && !inputs.epics.is_empty() {
            cuts.push("epics\u{2192}count".to_owned());
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

    let ready_shown = inputs.ready.len().min(SECTION_ROWS);
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
        epics: inputs.epics,
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

/// The one spelling of a per-type tally, shared by every surface that
/// prints one.
#[must_use]
pub fn counts_phrase(counts: &Counts) -> String {
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
    render_in_flight(&mut out, &inputs.in_flight, ladder);
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
    render_epics(&mut out, &inputs.epics, ladder);
    render_debt(&mut out, &inputs.debt, ladder);
    out
}

/// What is in motion, and where the first of it stopped. The floor keeps
/// that one line — a session cannot resume without it — and counts the
/// rest.
fn render_in_flight(out: &mut String, in_flight: &[ActiveTask], ladder: Ladder) {
    let shown = if ladder.reached(Collapse::Floor) {
        1
    } else {
        SECTION_ROWS
    }
    .min(in_flight.len());
    for (position, task) in in_flight.iter().take(shown).enumerate() {
        let _ = writeln!(out, "in-flight: {} {}", task.id, json_quoted(&task.title));
        if position == 0
            && !ladder.reached(Collapse::Log)
            && let Some(log) = &task.log
        {
            let _ = writeln!(out, "log: {log}");
        }
    }
    if in_flight.len() > shown {
        let _ = writeln!(out, "  \u{2026} {} more in flight", in_flight.len() - shown);
    }
}

/// What a bounded section left unsaid. The rows above name what they are,
/// so the count needs no noun — and needs no plural it would get wrong at
/// one.
fn section_hint(out: &mut String, total: usize, shown: usize) {
    if total > shown {
        let _ = writeln!(out, "  \u{2026} {} more", total - shown);
    }
}

/// Where each epic stands, so a session that opens with "continue <epic>"
/// can see which one it means and what it would pick up.
fn render_epics(out: &mut String, epics: &[Epic], ladder: Ladder) {
    if epics.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Epics) {
        let _ = writeln!(out, "epics: {} — anb list --for <id>", epics.len());
        return;
    }
    let _ = writeln!(out, "epics[{}]:", epics.len());
    for epic in epics.iter().take(SECTION_ROWS) {
        let _ = writeln!(out, "  {}", epic_line(epic));
    }
    section_hint(out, epics.len(), SECTION_ROWS);
}

/// Where one epic stands, in the one spelling every surface that prints an
/// epic uses. Progress, then what to do about it: the next Task to pick up,
/// the acceptance close the hub is now waiting for, or the plain fact that
/// nothing inside is dispatchable.
#[must_use]
pub fn epic_line(epic: &Epic) -> String {
    let progress = format!("{}: {}/{} closed", epic.id, epic.closed, epic.total);
    match &epic.next {
        Some(next) => format!("{progress}, next: {next}"),
        None if epic.closed == epic.total => {
            format!("{progress} — awaiting its acceptance close")
        }
        None => format!("{progress} — nothing ready"),
    }
}

fn render_review(out: &mut String, review: &[String], ladder: Ladder) {
    if review.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Log) {
        let _ = writeln!(out, "review: {}", review.len());
        return;
    }
    let _ = writeln!(
        out,
        "review[{}]: {} — waiting on a human",
        review.len(),
        encode::id_list(review, SECTION_ROWS)
    );
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
    for rule in rules.iter().take(SECTION_ROWS) {
        let _ = writeln!(out, "  {}: {}", rule.id, rule.title);
    }
    section_hint(out, rules.len(), SECTION_ROWS);
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

/// The Debt list, class by class, each bounded on its own so one crowded
/// class cannot bury the rest.
fn render_debt(out: &mut String, debt: &[DebtSignal], ladder: Ladder) {
    if debt.is_empty() {
        return;
    }
    if ladder.reached(Collapse::Debt) {
        let _ = writeln!(out, "debt: {}", debt.len());
        return;
    }
    let _ = writeln!(out, "debt[{}]:", debt.len());
    for class in debt_classes(debt) {
        for signal in &class.shown {
            let _ = writeln!(out, "  {}", signal.line());
        }
        section_hint(out, class.held, SECTION_ROWS);
    }
}

/// One Debt class as the dashboard shows it: the head, and how many the
/// class holds behind it.
pub struct DebtClass<'a> {
    pub shown: Vec<&'a DebtSignal>,
    pub held: usize,
}

/// The Debt classes the dashboard shows, in the order the list first names
/// them, whether or not a class's signals arrive together. Debt is the one
/// section bounded per class rather than as a whole, so both renderings ask
/// here instead of each deriving the shape from the row bound.
#[must_use]
pub fn debt_classes(debt: &[DebtSignal]) -> Vec<DebtClass<'_>> {
    let mut classes: Vec<DebtClass<'_>> = Vec::new();
    let mut named: Vec<&'static str> = Vec::new();
    for signal in debt {
        if named.contains(&signal.code()) {
            continue;
        }
        named.push(signal.code());
        let class = debt.iter().filter(|other| other.code() == signal.code());
        classes.push(DebtClass {
            shown: class.clone().take(SECTION_ROWS).collect(),
            held: class.count(),
        });
    }
    classes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a hub's line says about where it stands: the next thing to pick
    /// up, an acceptance close to give, or work that is there but blocked.
    #[test]
    fn an_epic_line_names_what_the_hub_is_waiting_for() {
        for (closed, total, next, expected) in [
            (
                1,
                3,
                Some("task.child"),
                "task.hub: 1/3 closed, next: task.child",
            ),
            (
                3,
                3,
                None,
                "task.hub: 3/3 closed \u{2014} awaiting its acceptance close",
            ),
            (1, 3, None, "task.hub: 1/3 closed \u{2014} nothing ready"),
        ] {
            let epic = Epic {
                id: "task.hub".to_owned(),
                closed,
                total,
                next: next.map(str::to_owned),
            };
            assert_eq!(epic_line(&epic), expected);
        }
    }
}
