//! The record model: the four types, their per-type vocabularies, and the
//! Task state machine.
//!
//! Types follow the persistence rule — how a record may change, not what it
//! is about: a Task closes through its workflow, a Decision dies
//! only by supersession or retirement, a Note is corrected in place, a
//! Question closes only by routing. This module judges one record at a time;
//! rules that need a second record live in the notebook.

use crate::finding::{Finding, FindingCode, Severity};
use crate::grammar::{self, RecordFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Task,
    Decision,
    Note,
    Question,
}

impl RecordType {
    pub const ALL: [RecordType; 4] = [
        RecordType::Task,
        RecordType::Decision,
        RecordType::Note,
        RecordType::Question,
    ];

    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|record_type| record_type.word() == word)
    }

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            RecordType::Task => "task",
            RecordType::Decision => "decision",
            RecordType::Note => "note",
            RecordType::Question => "question",
        }
    }

    /// The live directory under the notebook root, e.g. `tasks`.
    #[must_use]
    pub fn directory(self) -> &'static str {
        match self {
            RecordType::Task => "tasks",
            RecordType::Decision => "decisions",
            RecordType::Note => "notes",
            RecordType::Question => "questions",
        }
    }

    #[must_use]
    pub fn initial_state(self) -> &'static str {
        match self {
            RecordType::Task | RecordType::Question => "open",
            RecordType::Decision | RecordType::Note => "active",
        }
    }

    #[must_use]
    pub fn states(self) -> &'static [&'static str] {
        match self {
            RecordType::Task => &["open", "active", "review", "closed"],
            RecordType::Decision => &["active", "superseded", "retired"],
            RecordType::Note => &["active", "retired"],
            RecordType::Question => &["open", "routed", "dropped"],
        }
    }

    /// The states in which a record still binds: an open piece of work, a
    /// standing rule, current knowledge, an unanswered doubt.
    #[must_use]
    pub fn live_states(self) -> &'static [&'static str] {
        match self {
            RecordType::Task => &["open", "active", "review"],
            RecordType::Decision | RecordType::Note => &["active"],
            RecordType::Question => &["open"],
        }
    }

    /// The kind words this type allows; `None` for a type that carries no
    /// `kind` in this format version.
    #[must_use]
    pub fn kinds(self) -> Option<&'static [&'static str]> {
        match self {
            RecordType::Decision => Some(&["rule", "shape", "drift"]),
            RecordType::Note => Some(&["fact", "term", "guide"]),
            RecordType::Task | RecordType::Question => None,
        }
    }
}

/// The Task workflow states: `open → active → review → closed`, review
/// optional, reopen explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Open,
    Active,
    Review,
    Closed,
}

impl TaskState {
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "open" => Some(TaskState::Open),
            "active" => Some(TaskState::Active),
            "review" => Some(TaskState::Review),
            "closed" => Some(TaskState::Closed),
            _ => None,
        }
    }

    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            TaskState::Open => "open",
            TaskState::Active => "active",
            TaskState::Review => "review",
            TaskState::Closed => "closed",
        }
    }

    /// Decide what `action` comes to from this state, before any byte moves.
    /// An invalid transition answers with the actions this state does allow,
    /// so no caller has to learn the state machine.
    ///
    /// # Errors
    /// The valid actions from this state, when `action` is not among them.
    pub(crate) fn transition(self, action: TaskAction) -> Result<Transition, Vec<TaskAction>> {
        if action.already_state() == Some(self) {
            return Ok(Transition::Already);
        }
        if action.sources().contains(&self) {
            return Ok(Transition::Move {
                from: self,
                to: action.target(),
            });
        }
        Err(TaskAction::ALL
            .into_iter()
            .filter(|action| action.sources().contains(&self))
            .collect())
    }
}

/// The Task transitions, named by their commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskAction {
    Start,
    Submit,
    Close,
    Return,
    Reopen,
}

impl TaskAction {
    pub(crate) const ALL: [TaskAction; 5] = [
        TaskAction::Start,
        TaskAction::Submit,
        TaskAction::Close,
        TaskAction::Return,
        TaskAction::Reopen,
    ];

    #[must_use]
    pub(crate) fn word(self) -> &'static str {
        match self {
            TaskAction::Start => "start",
            TaskAction::Submit => "submit",
            TaskAction::Close => "close",
            TaskAction::Return => "return",
            TaskAction::Reopen => "reopen",
        }
    }

    fn target(self) -> TaskState {
        match self {
            TaskAction::Start | TaskAction::Return => TaskState::Active,
            TaskAction::Submit => TaskState::Review,
            TaskAction::Close => TaskState::Closed,
            TaskAction::Reopen => TaskState::Open,
        }
    }

    /// The state that proves this action already happened, making its replay
    /// safe. `Return` has none: an active Task may simply never have been
    /// submitted, and reporting that as a replayed return would hide a
    /// forbidden move.
    fn already_state(self) -> Option<TaskState> {
        match self {
            TaskAction::Start => Some(TaskState::Active),
            TaskAction::Submit => Some(TaskState::Review),
            TaskAction::Close => Some(TaskState::Closed),
            TaskAction::Reopen => Some(TaskState::Open),
            TaskAction::Return => None,
        }
    }

    fn sources(self) -> &'static [TaskState] {
        match self {
            TaskAction::Start => &[TaskState::Open],
            TaskAction::Submit => &[TaskState::Active],
            TaskAction::Close => &[TaskState::Active, TaskState::Review],
            TaskAction::Return => &[TaskState::Review],
            TaskAction::Reopen => &[TaskState::Closed],
        }
    }
}

/// What a transition request comes to: a state move, or a replay that must
/// change no byte (`already: true` at the reply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Transition {
    Move { from: TaskState, to: TaskState },
    Already,
}

/// One parsed record: the file, where it sits, and every finding the record
/// alone can produce — lexical, placement, and semantic.
pub struct Record {
    path: String,
    file: RecordFile,
    findings: Vec<Finding>,
}

impl Record {
    /// Parse `text` as the record at the notebook-relative `path`.
    #[must_use]
    pub fn parse(path: &str, text: &str) -> Self {
        let file = RecordFile::parse(text);
        let mut findings = file.findings().to_vec();
        findings.extend(file.placement_findings(path));
        findings.extend(semantic_findings(&file));
        Record {
            path: path.to_owned(),
            file,
            findings,
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn file(&self) -> &RecordFile {
        &self.file
    }

    /// Surrender the file for mutation; the findings die with the view, as
    /// they describe the bytes before the splice.
    #[must_use]
    pub(crate) fn into_file(self) -> RecordFile {
        self.file
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.code.severity() == Severity::Error)
    }

    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.file.field("id")
    }

    #[must_use]
    pub fn record_type(&self) -> Option<RecordType> {
        RecordType::from_word(self.file.field("type")?)
    }

    /// The raw state word; each caller types it against its record type.
    #[must_use]
    pub fn state(&self) -> Option<&str> {
        self.file.field("state")
    }

    /// Whether the record still binds: its state is among its type's live
    /// states.
    #[must_use]
    pub fn is_live(&self) -> bool {
        match (self.record_type(), self.state()) {
            (Some(record_type), Some(state)) => record_type.live_states().contains(&state),
            _ => false,
        }
    }

    /// The Origin: the record this one was born from (`from`).
    #[must_use]
    pub fn origin(&self) -> Option<&str> {
        self.file.field("from")
    }

    #[must_use]
    pub fn supersedes(&self) -> Option<&str> {
        self.file.field("supersedes")
    }

    #[must_use]
    pub fn superseded_by(&self) -> Option<&str> {
        self.file.field("superseded-by")
    }

    #[must_use]
    pub fn routed_to(&self) -> Option<&str> {
        self.file.field("routed-to")
    }

    #[must_use]
    pub fn hold(&self) -> Option<&str> {
        self.file.field("hold")
    }

    #[must_use]
    pub fn hold_until(&self) -> Option<&str> {
        self.file.field("hold-until")
    }
}

/// Fields legal only on some types; elsewhere they are orphans.
const TYPE_BOUND_FIELDS: &[(&str, &[RecordType])] = &[
    ("priority", &[RecordType::Task]),
    ("hold", &[RecordType::Task]),
    ("hold-until", &[RecordType::Task]),
    ("blocked-by", &[RecordType::Task]),
    ("routed-to", &[RecordType::Question]),
];

/// The record model's pass over one parsed file: per-type state and kind
/// vocabularies, field applicability, and the structural-close guarantees
/// one record can carry alone.
fn semantic_findings(file: &RecordFile) -> Vec<Finding> {
    let Some(record_type) = file.field("type").and_then(RecordType::from_word) else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    check_state(record_type, file, &mut findings);
    check_kind(record_type, file, &mut findings);
    check_type_bound_fields(record_type, file, &mut findings);
    check_hold_pairing(record_type, file, &mut findings);
    check_routing(record_type, file, &mut findings);
    check_dependencies(record_type, file, &mut findings);
    findings
}

fn check_state(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    let Some((value, line)) = file.field_entry("state") else {
        return;
    };
    if is_flagged_by_grammar(value) || record_type.states().contains(&value) {
        return;
    }
    let message = format!(
        "state: `{value}` is not one of {} for a {}",
        record_type.states().join(", "),
        record_type.word()
    );
    findings.push(Finding::located(line, FindingCode::BadValue, message));
}

fn check_kind(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    let Some((value, line)) = file.field_entry("kind") else {
        return;
    };
    let Some(kinds) = record_type.kinds() else {
        let message = format!("kind: a {} carries no kind", record_type.word());
        findings.push(Finding::located(line, FindingCode::UnknownField, message));
        return;
    };
    if is_flagged_by_grammar(value) || kinds.contains(&value) {
        return;
    }
    let message = format!(
        "kind: `{value}` is not one of {} for a {}",
        kinds.join(", "),
        record_type.word()
    );
    findings.push(Finding::located(line, FindingCode::BadValue, message));
}

/// The grammar already rejects a value that is not one lowercase word;
/// naming the enum on top of that would flag one defect twice.
fn is_flagged_by_grammar(value: &str) -> bool {
    !grammar::is_lower_word(value)
}

fn check_type_bound_fields(
    record_type: RecordType,
    file: &RecordFile,
    findings: &mut Vec<Finding>,
) {
    for (key, applies_to) in TYPE_BOUND_FIELDS {
        if applies_to.contains(&record_type) {
            continue;
        }
        if let Some((_, line)) = file.field_entry(key) {
            let message = format!("{key}: does not apply to a {}", record_type.word());
            findings.push(Finding::located(line, FindingCode::OrphanField, message));
        }
    }
}

/// On a non-Task the whole field is already an orphan; naming the missing
/// pair too would report one defect twice.
fn check_hold_pairing(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    if record_type != RecordType::Task {
        return;
    }
    if let Some((_, line)) = file.field_entry("hold-until")
        && file.field_entry("hold").is_none()
    {
        let message = "hold-until: legal only beside `hold`".to_owned();
        findings.push(Finding::located(line, FindingCode::OrphanField, message));
    }
}

/// A dependency edge names what must close first, so it can only point at a
/// Task — the self-describing id prefix lets one record judge that alone.
/// A record waiting on itself is the one cycle a single file can carry;
/// longer cycles are the notebook's to find.
fn check_dependencies(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    if record_type != RecordType::Task {
        return;
    }
    for (target, line) in file.field_entries("blocked-by") {
        if grammar::id_error(target).is_some() {
            continue;
        }
        if !target.starts_with("task.") {
            let message = format!("blocked-by: a task waits on a task, not `{target}`");
            findings.push(Finding::located(line, FindingCode::BadValue, message));
        } else if file.field("id") == Some(target) {
            let message = format!("blocked-by: `{target}` waits on itself");
            findings.push(Finding::located(line, FindingCode::DepCycle, message));
        }
    }
}

/// A Question recorded as routed must carry the thread of what closed it;
/// whether the thread's far end exists is the notebook's to verify.
fn check_routing(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    if record_type != RecordType::Question {
        return;
    }
    if let Some((target, line)) = file.field_entry("routed-to")
        && grammar::id_error(target).is_none()
        && let Some(target_type) = target
            .split_once('.')
            .and_then(|(word, _)| RecordType::from_word(word))
        && !matches!(target_type, RecordType::Decision | RecordType::Task)
    {
        let message = format!(
            "routed-to: a question routes into a decision or a task, not a {}",
            target_type.word()
        );
        findings.push(Finding::located(line, FindingCode::BrokenRouting, message));
    }
    let Some((state, line)) = file.field_entry("state") else {
        return;
    };
    if state == "routed" && file.field_entry("routed-to").is_none() {
        let message = "state: `routed` without `routed-to`".to_owned();
        findings.push(Finding::located(line, FindingCode::BrokenRouting, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Severity;

    fn record_text(field_lines: &[&str], body: &str) -> String {
        let mut text = String::from("---\n");
        for line in field_lines {
            text.push_str(line);
            text.push('\n');
        }
        text.push_str("---\n");
        text.push_str(body);
        text
    }

    fn question(state_line: &str, extra: &[&str]) -> Record {
        let mut lines = vec![
            "id: question.demo",
            "type: question",
            state_line,
            "title: A demo question",
            "created: 2026-08-24",
        ];
        lines.extend_from_slice(extra);
        Record::parse("questions/question.demo.md", &record_text(&lines, ""))
    }

    fn task_with(extra: &[&str]) -> Record {
        let mut lines = vec![
            "id: task.demo",
            "type: task",
            "state: open",
            "title: A demo task",
            "created: 2026-08-24",
        ];
        lines.extend_from_slice(extra);
        Record::parse("tasks/task.demo.md", &record_text(&lines, ""))
    }

    fn codes(record: &Record) -> Vec<FindingCode> {
        record
            .findings()
            .iter()
            .map(|finding| finding.code)
            .collect()
    }

    #[test]
    fn every_record_type_parses_clean_with_its_initial_state() {
        for record_type in RecordType::ALL {
            let word = record_type.word();
            let lines = [
                format!("id: {word}.demo"),
                format!("type: {word}"),
                format!("state: {}", record_type.initial_state()),
                "title: A demo record".to_owned(),
                "created: 2026-08-24".to_owned(),
            ];
            let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
            let path = format!("{}/{word}.demo.md", record_type.directory());
            let record = Record::parse(&path, &record_text(&lines, ""));
            assert_eq!(record.findings(), &[], "{word} must parse clean");
        }
    }

    #[test]
    fn a_state_outside_the_types_enum_names_the_valid_set() {
        let record = Record::parse(
            "tasks/task.demo.md",
            &record_text(
                &[
                    "id: task.demo",
                    "type: task",
                    "state: routed",
                    "title: A demo task",
                    "created: 2026-08-24",
                ],
                "",
            ),
        );
        assert_eq!(codes(&record), vec![FindingCode::BadValue]);
        assert!(
            record.findings()[0]
                .message
                .contains("open, active, review, closed"),
            "message must name the valid set: {}",
            record.findings()[0].message
        );
        assert!(record.has_errors());
    }

    #[test]
    fn a_kind_on_a_kindless_type_is_a_forward_compatible_warning() {
        let record = task_with(&["kind: feature"]);
        assert_eq!(codes(&record), vec![FindingCode::UnknownField]);
        assert!(!record.has_errors(), "the record stays fully usable");
    }

    #[test]
    fn a_kind_outside_the_types_enum_names_the_valid_set() {
        let record = Record::parse(
            "decisions/decision.demo.md",
            &record_text(
                &[
                    "id: decision.demo",
                    "type: decision",
                    "state: active",
                    "kind: law",
                    "title: A demo decision",
                    "created: 2026-08-24",
                ],
                "",
            ),
        );
        assert_eq!(codes(&record), vec![FindingCode::BadValue]);
        assert!(record.findings()[0].message.contains("rule, shape, drift"));
    }

    #[test]
    fn a_task_only_field_on_another_type_is_an_orphan_warning() {
        let record = Record::parse(
            "notes/note.demo.md",
            &record_text(
                &[
                    "id: note.demo",
                    "type: note",
                    "state: active",
                    "title: A demo note",
                    "priority: 2",
                    "created: 2026-08-24",
                ],
                "",
            ),
        );
        assert_eq!(codes(&record), vec![FindingCode::OrphanField]);
        assert!(!record.has_errors());
    }

    #[test]
    fn a_field_orphaned_twice_over_is_named_once() {
        let record = Record::parse(
            "notes/note.demo.md",
            &record_text(
                &[
                    "id: note.demo",
                    "type: note",
                    "state: active",
                    "title: A note with an unpaired task date",
                    "hold-until: 2026-09-10",
                    "created: 2026-08-24",
                ],
                "",
            ),
        );
        assert_eq!(codes(&record), vec![FindingCode::OrphanField]);
    }

    #[test]
    fn a_return_needs_a_review_to_return_from() {
        assert_eq!(
            TaskState::Active.transition(TaskAction::Return),
            Err(vec![TaskAction::Submit, TaskAction::Close])
        );
    }

    #[test]
    fn hold_until_without_hold_is_an_orphan_warning() {
        let record = task_with(&["hold-until: 2026-09-01"]);
        assert_eq!(codes(&record), vec![FindingCode::OrphanField]);
    }

    #[test]
    fn hold_until_beside_hold_is_clean() {
        let record = task_with(&[
            "hold: waiting for the 1.99 release",
            "hold-until: 2026-09-01",
        ]);
        assert_eq!(record.findings(), &[]);
        assert_eq!(record.hold(), Some("waiting for the 1.99 release"));
        assert_eq!(record.hold_until(), Some("2026-09-01"));
    }

    #[test]
    fn a_question_routed_without_routed_to_is_broken_routing() {
        let record = question("state: routed", &[]);
        assert_eq!(codes(&record), vec![FindingCode::BrokenRouting]);
        assert_eq!(
            record.findings()[0].code.severity(),
            Severity::Error,
            "a lost routing thread excludes the record from mutation"
        );
    }

    #[test]
    fn a_question_routed_with_routed_to_is_clean_for_this_record_alone() {
        let record = question("state: routed", &["routed-to: decision.the-answer"]);
        assert_eq!(record.findings(), &[]);
        assert_eq!(record.routed_to(), Some("decision.the-answer"));
    }

    #[test]
    fn a_question_routed_into_a_type_no_answer_becomes_is_broken_routing() {
        let record = question("state: routed", &["routed-to: note.a-fact"]);
        assert_eq!(codes(&record), vec![FindingCode::BrokenRouting]);
        assert!(
            record.findings()[0].message.contains("decision or a task"),
            "the message names where a question may route: {}",
            record.findings()[0].message
        );
    }

    #[test]
    fn a_malformed_state_is_flagged_once_not_twice() {
        let record = Record::parse(
            "tasks/task.demo.md",
            &record_text(
                &[
                    "id: task.demo",
                    "type: task",
                    "state: Open",
                    "title: A demo task",
                    "created: 2026-08-24",
                ],
                "",
            ),
        );
        assert_eq!(codes(&record), vec![FindingCode::BadValue]);
    }

    #[test]
    fn the_task_machine_moves_along_the_spec_table() {
        use TaskAction::{Close, Reopen, Return, Start, Submit};
        use TaskState::{Active, Closed, Open, Review};
        let moves = [
            (Open, Start, Active),
            (Active, Submit, Review),
            (Active, Close, Closed),
            (Review, Close, Closed),
            (Review, Return, Active),
            (Closed, Reopen, Open),
        ];
        for (from, action, to) in moves {
            assert_eq!(
                from.transition(action),
                Ok(Transition::Move { from, to }),
                "{} from {}",
                action.word(),
                from.word()
            );
        }
    }

    #[test]
    fn reaching_the_state_the_action_targets_is_a_replay_not_an_error() {
        assert_eq!(
            TaskState::Active.transition(TaskAction::Start),
            Ok(Transition::Already)
        );
        assert_eq!(
            TaskState::Closed.transition(TaskAction::Close),
            Ok(Transition::Already)
        );
    }

    #[test]
    fn an_invalid_transition_answers_with_the_valid_actions() {
        assert_eq!(
            TaskState::Open.transition(TaskAction::Close),
            Err(vec![TaskAction::Start])
        );
        assert_eq!(
            TaskState::Review.transition(TaskAction::Start),
            Err(vec![TaskAction::Close, TaskAction::Return])
        );
    }
}
