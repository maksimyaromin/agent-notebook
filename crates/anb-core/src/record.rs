//! The record model: the four types, their per-type vocabularies, and the
//! Task state machine.
//!
//! Types follow the persistence rule — how a record may change, not what it
//! is about: a Task closes through its workflow, a Decision dies
//! only by supersession or retirement, a Note is corrected in place, a
//! Question closes like a Task. This module judges one record at a time;
//! rules that need a second record live in the notebook.

use crate::finding::{Finding, FindingCode};
use crate::grammar::{self, RecordFile, Residence};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    Task,
    Decision,
    Note,
    Question,
}

/// The directory history moves into, under the notebook root. Every type's
/// archive is a directory inside it.
pub const ARCHIVE_DIR: &str = "archive";

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
            RecordType::Question => &["open", "closed"],
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
/// optional, reopen explicit; a close by reason reaches `closed` from any
/// live state.
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
        if action.already_state() == self {
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

/// The Task transitions, named as an agent asks for them. `start` also
/// takes a Task back from review. `CloseWithReason` is `close --reason`:
/// the Task ends without work, so it may end from `open`, where a close
/// carrying a proof is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskAction {
    Start,
    Submit,
    Close,
    Reopen,
    CloseWithReason,
}

impl TaskAction {
    pub(crate) const ALL: [TaskAction; 5] = [
        TaskAction::Start,
        TaskAction::Submit,
        TaskAction::Close,
        TaskAction::Reopen,
        TaskAction::CloseWithReason,
    ];

    /// The move as the agent types it, which is also the key its retry
    /// shape is filed under.
    #[must_use]
    pub(crate) fn word(self) -> &'static str {
        match self {
            TaskAction::Start => "start",
            TaskAction::Submit => "submit",
            TaskAction::Close => "close",
            TaskAction::Reopen => "reopen",
            TaskAction::CloseWithReason => "close --reason",
        }
    }

    fn target(self) -> TaskState {
        match self {
            TaskAction::Start => TaskState::Active,
            TaskAction::Submit => TaskState::Review,
            TaskAction::Close | TaskAction::CloseWithReason => TaskState::Closed,
            TaskAction::Reopen => TaskState::Open,
        }
    }

    /// The state that proves this action already happened, making its replay
    /// safe.
    fn already_state(self) -> TaskState {
        match self {
            TaskAction::Start => TaskState::Active,
            TaskAction::Submit => TaskState::Review,
            TaskAction::Close | TaskAction::CloseWithReason => TaskState::Closed,
            TaskAction::Reopen => TaskState::Open,
        }
    }

    fn sources(self) -> &'static [TaskState] {
        match self {
            TaskAction::Start => &[TaskState::Open, TaskState::Review],
            TaskAction::Submit => &[TaskState::Active],
            TaskAction::Close => &[TaskState::Active, TaskState::Review],
            TaskAction::Reopen => &[TaskState::Closed],
            TaskAction::CloseWithReason => &[TaskState::Open, TaskState::Active, TaskState::Review],
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
        findings.extend(residence_finding(path, &file));
        Record {
            path: path.to_owned(),
            file,
            findings,
        }
    }

    /// The record at `path` whose bytes could not cross the Storage seam:
    /// no envelope and no body, carrying only the `not-utf8` finding, so the
    /// file stays visible as invalid instead of aborting the command that
    /// met it. Having no envelope, it is a record no verb can splice.
    #[must_use]
    pub fn unreadable(path: &str) -> Self {
        Record {
            path: path.to_owned(),
            file: RecordFile::parse(""),
            findings: vec![not_utf8_finding()],
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

    /// The well-formed ids a Task waits on. A malformed target is a
    /// finding `check` names, never an edge.
    pub(crate) fn blocked_by(&self) -> impl Iterator<Item = &str> {
        self.file
            .field_values("blocked-by")
            .filter(|target| grammar::id_error(target).is_none())
    }

    #[must_use]
    pub fn error_findings(&self) -> Vec<Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.is_error())
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(Finding::is_error)
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
    pub fn resolved_by(&self) -> Option<&str> {
        self.file.field("resolved-by")
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

/// The finding a file that is not UTF-8 carries, wherever a read meets one.
pub(crate) fn not_utf8_finding() -> Finding {
    Finding::for_file(
        FindingCode::NotUtf8,
        "the file is not valid UTF-8".to_owned(),
    )
}

/// Fields legal only on some types; elsewhere they are orphans.
const TYPE_BOUND_FIELDS: &[(&str, &[RecordType])] = &[
    ("priority", &[RecordType::Task]),
    ("hold", &[RecordType::Task]),
    ("hold-until", &[RecordType::Task]),
    ("blocked-by", &[RecordType::Task]),
    ("resolved-by", &[RecordType::Question]),
    ("reason", &[RecordType::Task, RecordType::Question]),
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
    check_resolution(record_type, file, &mut findings);
    check_dependencies(record_type, file, &mut findings);
    findings
}

/// The state word says whether a record still binds; its directory says
/// whether it is history. The two may disagree only as a named finding.
///
/// Reading is not the axis — the archive is history, and history is meant
/// to be readable. Moving is: a verb that acts on an existing record
/// resolves its id against the live directory and refuses an archived one.
/// So a record that binds from inside the archive cannot be settled,
/// corrected, or filed until `restore` brings it back, while a settled
/// record still in the working set is one command from its home. That gap
/// is the whole severity split.
fn residence_finding(path: &str, file: &RecordFile) -> Option<Finding> {
    let type_word = file.field("type")?;
    let record_type = RecordType::from_word(type_word)?;
    let (state, line) = file.field_entry("state")?;
    if !record_type.states().contains(&state) {
        return None;
    }
    let binds = record_type.live_states().contains(&state);
    let directory = record_type.directory();
    match (binds, grammar::residence(path, type_word)?) {
        (true, Residence::Archive) => Some(Finding::located(
            line,
            FindingCode::ArchivedLiveRecord,
            format!(
                "state: `{state}` still binds, but the file sits in `archive/{directory}/` — every verb that would move it on refuses an archived id"
            ),
        )),
        (false, Residence::Live) => Some(Finding::located(
            line,
            FindingCode::UnarchivedSettledRecord,
            format!("state: `{state}` is settled, but the file still sits in `{directory}/`"),
        )),
        _ => None,
    }
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
/// A Task waiting on itself is named here rather than with the longer
/// cycles, so the mutation gate holds it: `unblock` runs over a finding on
/// a `blocked-by` line, so the edge that freezes the record is also the
/// one the record can still have erased.
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
            findings.push(Finding::located(line, FindingCode::BlockCycle, message));
        }
    }
}

/// A closed Question names what settled it: the Decision or Task it
/// resolved into, or the reason it closed without one. A reason on a record
/// that is not closed contradicts its own state. Whether a named resolver
/// exists is the notebook's to verify.
fn check_resolution(record_type: RecordType, file: &RecordFile, findings: &mut Vec<Finding>) {
    if let Some((target, line)) = file.field_entry("resolved-by")
        && grammar::id_error(target).is_none()
        && let Some(target_type) = target
            .split_once('.')
            .and_then(|(word, _)| RecordType::from_word(word))
        && !matches!(target_type, RecordType::Decision | RecordType::Task)
    {
        let message = format!(
            "resolved-by: a question resolves into a decision or a task, not a {}",
            target_type.word()
        );
        findings.push(Finding::located(line, FindingCode::BadValue, message));
    }
    let Some((state, line)) = file.field_entry("state") else {
        return;
    };
    if state != "closed" {
        if let Some((_, reason_line)) = file.field_entry("reason") {
            let message = format!("reason: only a closed record carries one, this one is {state}");
            findings.push(Finding::located(
                reason_line,
                FindingCode::BadValue,
                message,
            ));
        }
        return;
    }
    if record_type == RecordType::Question
        && file.field_entry("resolved-by").is_none()
        && file.field_entry("reason").is_none()
    {
        let message = "state: `closed` needs `resolved-by` or `reason`".to_owned();
        findings.push(Finding::located(line, FindingCode::MissingField, message));
    }
}

/// The envelope keys whose values point at other records.
pub(crate) const REF_KEYS: [&str; 5] = [
    "from",
    "supersedes",
    "superseded-by",
    "resolved-by",
    "blocked-by",
];

/// The record a `link` line points at, if it points at one at all.
///
/// A link is `<kind> <target>`, and its target is a pull request, a commit,
/// a path, or — since a close may carry its report as a Note — a record id.
/// Only the id-shaped target names a record; nothing else can be resolved,
/// and nothing else may be mistaken for a reference.
pub(crate) fn linked_record(link: &str) -> Option<&str> {
    let (_, target) = grammar::split_link(link)?;
    grammar::id_error(target).is_none().then_some(target)
}

/// The finding a reference into nothing deserves: one condition, one code,
/// on every surface.
pub(crate) fn dangling_finding(key: &str, target: &str, line: Option<usize>) -> Finding {
    Finding::located(
        line,
        FindingCode::DanglingRef,
        format!("{key}: `{target}` names no record"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// A question in its filed home: state and residence agree there, so a
    /// resolution finding is the only one these cases can produce.
    fn archived_question(state_line: &str, extra: &[&str]) -> Record {
        let mut lines = vec![
            "id: question.demo",
            "type: question",
            state_line,
            "title: A demo question",
            "created: 2026-08-24",
        ];
        lines.extend_from_slice(extra);
        Record::parse(
            "archive/questions/question.demo.md",
            &record_text(&lines, ""),
        )
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
    fn a_word_outside_its_types_vocabulary_names_the_whole_set() {
        for (fields, valid) in [
            (["state: active", "kind: law"], "rule, shape, drift"),
            (
                ["state: cancelled", "kind: rule"],
                "active, superseded, retired",
            ),
        ] {
            let mut lines = vec!["id: decision.demo", "type: decision"];
            lines.extend(fields);
            lines.extend(["title: A demo decision", "created: 2026-08-24"]);
            let record = Record::parse("decisions/decision.demo.md", &record_text(&lines, ""));
            assert_eq!(codes(&record), vec![FindingCode::BadValue], "{fields:?}");
            assert!(
                record.findings()[0].message.contains(valid),
                "{fields:?} must name `{valid}`: {}",
                record.findings()[0].message
            );
        }
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
    fn a_question_resolved_into_a_type_no_question_settles_into_is_a_bad_value() {
        let record = archived_question("state: closed", &["resolved-by: note.a-fact"]);
        assert_eq!(codes(&record), vec![FindingCode::BadValue]);
        assert!(
            record.findings()[0].message.contains("decision or a task"),
            "the message names where a question may resolve: {}",
            record.findings()[0].message
        );
    }

    #[test]
    fn a_closed_question_naming_neither_resolver_nor_reason_misses_a_field() {
        let record = archived_question("state: closed", &[]);
        assert_eq!(codes(&record), vec![FindingCode::MissingField]);
    }

    #[test]
    fn a_reason_on_a_record_that_is_not_closed_is_a_bad_value() {
        let record = archived_question("state: open", &["reason: moot"]);
        assert!(
            codes(&record).contains(&FindingCode::BadValue),
            "{:?}",
            codes(&record)
        );
    }

    fn record_at(directory: &str, record_type: RecordType, state: &str) -> Record {
        let word = record_type.word();
        let id = format!("{word}.demo");
        let lines = [
            format!("id: {id}"),
            format!("type: {word}"),
            format!("state: {state}"),
            "title: A demo record".to_owned(),
            "created: 2026-08-24".to_owned(),
        ];
        let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
        Record::parse(&format!("{directory}/{id}.md"), &record_text(&lines, ""))
    }

    #[test]
    fn every_binding_state_is_one_of_its_types_states() {
        for record_type in RecordType::ALL {
            for live in record_type.live_states() {
                assert!(
                    record_type.states().contains(live),
                    "{live} binds for a {} but is not one of its states",
                    record_type.word()
                );
            }
        }
    }

    #[test]
    fn a_state_and_a_home_that_agree_leave_the_residence_axis_silent() {
        for record_type in RecordType::ALL {
            for state in record_type.states() {
                let binds = record_type.live_states().contains(state);
                let directory = record_type.directory();
                let home = if binds {
                    directory.to_owned()
                } else {
                    format!("archive/{directory}")
                };
                let record = record_at(&home, record_type, state);
                assert!(
                    !record.findings().iter().any(|finding| matches!(
                        finding.code,
                        FindingCode::ArchivedLiveRecord | FindingCode::UnarchivedSettledRecord
                    )),
                    "{state} in {home}/ must not disagree: {:?}",
                    record.findings()
                );
            }
        }
    }

    #[test]
    fn every_state_in_the_wrong_home_is_named_by_its_direction() {
        for record_type in RecordType::ALL {
            for state in record_type.states() {
                let binds = record_type.live_states().contains(state);
                let directory = record_type.directory();
                let wrong = if binds {
                    format!("archive/{directory}")
                } else {
                    directory.to_owned()
                };
                let expected = if binds {
                    FindingCode::ArchivedLiveRecord
                } else {
                    FindingCode::UnarchivedSettledRecord
                };
                let record = record_at(&wrong, record_type, state);
                assert!(
                    codes(&record).contains(&expected),
                    "{state} in {wrong}/ must be named {expected}: {:?}",
                    record.findings()
                );
            }
        }
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
}
