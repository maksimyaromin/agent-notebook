//! The envelope line grammar: parse, render, normalize.
//!
//! A record file opens with a `---`-fenced envelope of single-line
//! `key: value` fields and continues as an opaque markdown body. Values are
//! typed by key through the field table, never guessed from their shape, so
//! implicit-typing corruption (`state: no` becoming a boolean) is
//! unrepresentable. Parsing is total: any input yields a [`RecordFile`] whose
//! [`render`](RecordFile::render) reproduces the input byte-exact; rejection
//! happens through named findings, never by dropping bytes.
//!
//! Reading and judging a file is anyone's; changing one is the notebook's,
//! whose write-time invariants bind whoever writes — so the splicing
//! methods stay inside the crate and a record is changed through a verb.

use crate::date;
use crate::finding::{Finding, FindingCode};

const FENCE: &str = "---";
const BOM: char = '\u{feff}';
const TYPE_WORDS: [&str; 4] = ["task", "decision", "note", "question"];

/// `task` → `tasks`: a record type names its directory by its plural.
fn type_directory(type_word: &str) -> Option<String> {
    TYPE_WORDS
        .contains(&type_word)
        .then(|| format!("{type_word}s"))
}

/// Which of a type's two homes holds a file: the working set, or the
/// archive that is history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Residence {
    Live,
    Archive,
}

/// The home `path` puts a record of `type_word` in, or `None` when the file
/// sits outside both — a placement the caller reports, never a residence.
/// Placement and residence answer from this one function, so a file can
/// never be judged misplaced and lodged somewhere at the same time.
pub(crate) fn residence(path: &str, type_word: &str) -> Option<Residence> {
    let directory = type_directory(type_word)?;
    let (parents, _) = path.rsplit_once('/')?;
    if parents == directory {
        Some(Residence::Live)
    } else if parents.strip_prefix("archive/") == Some(directory.as_str()) {
        Some(Residence::Archive)
    } else {
        None
    }
}

/// What a field's value must look like, checked by key.
#[derive(Clone, Copy)]
enum Form {
    /// `<type>.<slug>` per the id grammar: ASCII, at most 64 bytes.
    Id,
    /// One of the four record type words.
    TypeWord,
    /// One lowercase ASCII word; the per-type enums belong to the record model.
    LowerWord,
    /// Non-empty free text.
    NonEmptyText,
    /// Free text, possibly empty.
    Text,
    /// An integer `0`–`4`.
    Priority,
    /// Comma-separated `[a-z0-9-]+` tokens.
    TagList,
    /// A `<kind> <target>` pair: a token, then a non-empty rest of line.
    Link,
    /// `YYYY-MM-DD`, or an RFC 3339 timestamp.
    Date,
}

struct FieldSpec {
    key: &'static str,
    form: Form,
    required: bool,
    repeatable: bool,
}

const fn required(key: &'static str, form: Form) -> FieldSpec {
    FieldSpec {
        key,
        form,
        required: true,
        repeatable: false,
    }
}

const fn optional(key: &'static str, form: Form) -> FieldSpec {
    FieldSpec {
        key,
        form,
        required: false,
        repeatable: false,
    }
}

/// A repeatable field is optional: zero lines is a legal count.
const fn repeatable(key: &'static str, form: Form) -> FieldSpec {
    FieldSpec {
        key,
        form,
        required: false,
        repeatable: true,
    }
}

/// The envelope's field table, each key beside its kin (`via` after `by`,
/// the workflow fields before the dates); table position is canonical
/// order. `updated` is written by
/// every mutation, but a hand-made file may lack it, so reading does not
/// require it.
const FIELD_TABLE: &[FieldSpec] = &[
    required("id", Form::Id),
    required("type", Form::TypeWord),
    required("state", Form::LowerWord),
    optional("kind", Form::LowerWord),
    required("title", Form::NonEmptyText),
    optional("by", Form::Text),
    optional("via", Form::Text),
    optional("from", Form::Id),
    optional("tags", Form::TagList),
    repeatable("link", Form::Link),
    optional("supersedes", Form::Id),
    optional("superseded-by", Form::Id),
    repeatable("blocked-by", Form::Id),
    optional("resolved-by", Form::Id),
    optional("reason", Form::NonEmptyText),
    optional("priority", Form::Priority),
    optional("hold", Form::NonEmptyText),
    optional("hold-until", Form::Date),
    required("created", Form::Date),
    optional("updated", Form::Date),
    optional("closed", Form::Date),
    optional("review-by", Form::Date),
];

fn field_spec(key: &str) -> Option<&'static FieldSpec> {
    FIELD_TABLE.iter().find(|spec| spec.key == key)
}

fn canonical_rank(key: &str) -> usize {
    FIELD_TABLE
        .iter()
        .position(|spec| spec.key == key)
        .unwrap_or(usize::MAX)
}

struct FieldLine {
    key: String,
    value: String,
    raw: String,
    /// 1-based parse line; `None` on a line spliced in by a mutation.
    line: Option<usize>,
}

enum EnvelopeLine {
    Field(FieldLine),
    /// An unrecognized envelope line, preserved verbatim beside its
    /// `bad-envelope-line` finding.
    Raw(String),
}

struct Envelope {
    /// Fence lines keep their raw bytes: a lenient read accepts `---\r\n`.
    open_fence: String,
    lines: Vec<EnvelopeLine>,
    close_fence: Option<String>,
}

impl Envelope {
    fn fields(&self) -> impl Iterator<Item = &FieldLine> {
        self.lines.iter().filter_map(|line| match line {
            EnvelopeLine::Field(field) => Some(field),
            EnvelopeLine::Raw(_) => None,
        })
    }

    fn first(&self, key: &str) -> Option<&FieldLine> {
        self.fields().find(|field| field.key == key)
    }

    fn first_mut(&mut self, key: &str) -> Option<&mut FieldLine> {
        self.lines.iter_mut().find_map(|line| match line {
            EnvelopeLine::Field(field) if field.key == key => Some(field),
            _ => None,
        })
    }

    fn last_index_of(&self, key: &str) -> Option<usize> {
        self.lines
            .iter()
            .rposition(|line| matches!(line, EnvelopeLine::Field(field) if field.key == key))
    }
}

fn canonical_line(key: &str, value: &str) -> String {
    debug_assert!(
        !value.contains('\n'),
        "a field value is one line; multi-line content belongs in the body"
    );
    if value.is_empty() {
        format!("{key}:\n")
    } else {
        format!("{key}: {value}\n")
    }
}

fn field_line(key: &str, value: &str, raw: String) -> EnvelopeLine {
    EnvelopeLine::Field(FieldLine {
        key: key.to_owned(),
        value: value.to_owned(),
        raw,
        line: None,
    })
}

/// Where a new `key` line belongs: after the last field the canonical order
/// puts at or before it, else ahead of every field. An unknown key ranks
/// last, so it never pulls a known field ahead of its place — even when it
/// sits mid-envelope.
fn insertion_index(envelope: &Envelope, key: &str) -> usize {
    let rank = canonical_rank(key);
    envelope
        .lines
        .iter()
        .rposition(
            |line| matches!(line, EnvelopeLine::Field(field) if canonical_rank(&field.key) <= rank),
        )
        .map_or(0, |last_at_or_before| last_at_or_before + 1)
}

/// A parsed record file: the envelope, the opaque body, and the findings.
///
/// The body is everything after the close fence, verbatim to EOF; the grammar
/// never scans it, so no body line — including one that is exactly `---` —
/// can be misparsed, and no escape convention exists.
pub struct RecordFile {
    bom: bool,
    envelope: Option<Envelope>,
    body: String,
    findings: Vec<Finding>,
}

impl RecordFile {
    /// Parse `input` completely; never returns early and never drops bytes.
    #[must_use]
    pub fn parse(input: &str) -> Self {
        let (bom, text) = split_bom(input);
        let mut file = parse_structure(text);
        file.bom = bom;
        if bom {
            let message = "file starts with a byte order mark".to_owned();
            file.findings
                .insert(0, Finding::at(1, FindingCode::Bom, message));
        }
        if let Some(envelope) = &file.envelope {
            check_fields(envelope, &mut file.findings);
        }
        file
    }

    /// Reproduce the parsed input byte-exact, whatever the findings were.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        if self.bom {
            out.push(BOM);
        }
        if let Some(envelope) = &self.envelope {
            out.push_str(&envelope.open_fence);
            for line in &envelope.lines {
                out.push_str(match line {
                    EnvelopeLine::Field(field) => &field.raw,
                    EnvelopeLine::Raw(raw) => raw,
                });
            }
            if let Some(close_fence) = &envelope.close_fence {
                out.push_str(close_fence);
            }
        }
        out.push_str(&self.body);
        out
    }

    /// The canonical form: fields in canonical order, one space after `:`,
    /// LF endings, no BOM — body verbatim. A file rejected with errors is
    /// never rewritten, so its canonical form is its own bytes.
    #[must_use]
    pub fn normalize(&self) -> String {
        if self.has_errors() {
            return self.render();
        }
        let Some(envelope) = &self.envelope else {
            return self.render();
        };
        let mut fields: Vec<&FieldLine> = envelope.fields().collect();
        fields.sort_by_key(|field| canonical_rank(&field.key));

        let mut out = String::from("---\n");
        for field in fields {
            out.push_str(&field.key);
            out.push(':');
            if !field.value.is_empty() {
                out.push(' ');
                out.push_str(&field.value);
            }
            out.push('\n');
        }
        out.push_str("---\n");
        out.push_str(&self.body);
        out
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(Finding::is_error)
    }

    /// Everything after the close fence, verbatim.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// The first occurrence's value; repeatable fields have [`Self::field_values`].
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&str> {
        self.envelope
            .as_ref()?
            .first(key)
            .map(|field| field.value.as_str())
    }

    /// Every occurrence's value, in file order.
    pub fn field_values<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.field_entries(key).map(|(value, _)| value)
    }

    /// Every field's key and value, in file order — the single-record
    /// read surface, which shows the envelope as it stands.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.envelope
            .iter()
            .flat_map(Envelope::fields)
            .map(|field| (field.key.as_str(), field.value.as_str()))
    }

    /// The first occurrence's value and line.
    pub(crate) fn field_entry(&self, key: &str) -> Option<(&str, Option<usize>)> {
        self.envelope
            .as_ref()?
            .first(key)
            .map(|field| (field.value.as_str(), field.line))
    }

    /// Every occurrence's value and line, in file order.
    pub(crate) fn field_entries<'a>(
        &'a self,
        key: &'a str,
    ) -> impl Iterator<Item = (&'a str, Option<usize>)> {
        self.envelope
            .iter()
            .flat_map(Envelope::fields)
            .filter(move |field| field.key == key)
            .map(|field| (field.value.as_str(), field.line))
    }

    /// Whether the file opens with a `---` fence, and so has an envelope to
    /// splice at all. The mutating methods below panic without one.
    pub(crate) fn has_envelope(&self) -> bool {
        self.envelope.is_some()
    }

    /// Replace `key`'s line with its canonical form, or insert a new line at
    /// the key's canonical position; every other byte of the file stays
    /// verbatim, so splicing into a CRLF file leaves its untouched lines
    /// CRLF while the spliced line is canonical LF. Returns whether any
    /// byte changed.
    ///
    /// The value must be one line — multi-line content belongs in the body.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn set_field(&mut self, key: &str, value: &str) -> bool {
        let canonical = canonical_line(key, value);
        let envelope = self.envelope_for_mutation();
        if let Some(field) = envelope.first_mut(key) {
            if field.raw == canonical {
                return false;
            }
            value.clone_into(&mut field.value);
            field.raw = canonical;
            return true;
        }
        let at = insertion_index(envelope, key);
        envelope.lines.insert(at, field_line(key, value, canonical));
        true
    }

    /// Add one more line of a repeatable `key` after its last occurrence
    /// (at the key's canonical position when it is the first). The value
    /// must be one line.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn append_field(&mut self, key: &str, value: &str) {
        let canonical = canonical_line(key, value);
        let envelope = self.envelope_for_mutation();
        let at = match envelope.last_index_of(key) {
            Some(last) => last + 1,
            None => insertion_index(envelope, key),
        };
        envelope.lines.insert(at, field_line(key, value, canonical));
    }

    /// Remove every line of `key`; every other byte stays verbatim.
    /// Returns whether any line was removed.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn remove_field(&mut self, key: &str) -> bool {
        let envelope = self.envelope_for_mutation();
        let before = envelope.lines.len();
        envelope
            .lines
            .retain(|line| !matches!(line, EnvelopeLine::Field(field) if field.key == key));
        envelope.lines.len() != before
    }

    /// Remove every line of a repeatable `key` carrying exactly `value`;
    /// every other byte stays verbatim. Returns whether any line was removed.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn remove_field_value(&mut self, key: &str, value: &str) -> bool {
        let envelope = self.envelope_for_mutation();
        let before = envelope.lines.len();
        envelope.lines.retain(|line| {
            !matches!(line, EnvelopeLine::Field(field) if field.key == key && field.value == value)
        });
        envelope.lines.len() != before
    }

    /// Append one line at EOF — the body's only mutation.
    /// A missing newline before the appended line is supplied, whether the
    /// file ended inside the envelope or mid-body-line.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn append_body(&mut self, line: &str) {
        if self.body.is_empty() {
            let close_fence = &mut self.envelope_for_mutation().close_fence;
            if let Some(fence) = close_fence
                && !fence.ends_with('\n')
            {
                fence.push('\n');
            }
        } else if !self.body.ends_with('\n') {
            self.body.push('\n');
        }
        self.body.push_str(line);
        self.body.push('\n');
    }

    /// Replace the body wholesale — the deliberate correction `edit` makes,
    /// as opposed to the log's [`Self::append_body`]. The envelope stays
    /// verbatim; a close fence left without its newline gains one so a
    /// non-empty body starts on its own line.
    ///
    /// # Panics
    /// On a file with no envelope; a caller mutates only accepted records.
    pub(crate) fn set_body(&mut self, body: &str) {
        if !body.is_empty() {
            let close_fence = &mut self.envelope_for_mutation().close_fence;
            if let Some(fence) = close_fence
                && !fence.ends_with('\n')
            {
                fence.push('\n');
            }
        }
        body.clone_into(&mut self.body);
    }

    fn envelope_for_mutation(&mut self) -> &mut Envelope {
        self.envelope
            .as_mut()
            .expect("a mutation runs only on a record with an envelope")
    }

    /// Check the record against where it sits: the filename must equal the id
    /// and the file must be in one of its type's two homes. `path` is
    /// notebook-relative (`tasks/task.x.md`, `archive/tasks/task.x.md`). A
    /// malformed id or type already carries its own finding and is not
    /// re-reported here.
    #[must_use]
    pub fn placement_findings(&self, path: &str) -> Vec<Finding> {
        let Some(envelope) = &self.envelope else {
            return Vec::new();
        };
        let (parents, filename) = split_path(path);
        let stem = filename.strip_suffix(".md").unwrap_or(filename);

        let mut findings = Vec::new();
        if let Some(id) = envelope.first("id")
            && id_error(&id.value).is_none()
            && id.value != stem
        {
            let message = format!("id `{}` does not match filename `{filename}`", id.value);
            findings.push(Finding::located(
                id.line,
                FindingCode::IdFilenameMismatch,
                message,
            ));
        }
        if let Some(parents) = parents
            && let Some(type_field) = envelope.first("type")
            && let Some(expected) = type_directory(&type_field.value)
            && residence(path, &type_field.value).is_none()
        {
            let message = format!(
                "type `{}` belongs under `{expected}/` or `archive/{expected}/`, not `{parents}/`",
                type_field.value
            );
            findings.push(Finding::located(
                type_field.line,
                FindingCode::TypeDirMismatch,
                message,
            ));
        }
        findings
    }
}

fn split_bom(input: &str) -> (bool, &str) {
    match input.strip_prefix(BOM) {
        Some(rest) => (true, rest),
        None => (false, input),
    }
}

/// The structural pass: fences, field lines, and the body split — every
/// finding the line grammar can name without knowing any field.
fn parse_structure(text: &str) -> RecordFile {
    let lines = text.split_inclusive('\n').collect::<Vec<&str>>();
    let opens_with_fence = lines.first().is_some_and(|first| {
        let (content, _) = line_content(first);
        content == FENCE
    });
    if !opens_with_fence {
        return file_without_envelope(text);
    }

    let scan = scan_envelope(lines[0], &lines[1..]);
    let body = text[scan.body_start..].to_owned();

    let mut findings = scan.findings;
    if scan.envelope.close_fence.is_none() {
        let message = "envelope has no closing `---` fence".to_owned();
        findings.push(Finding::at(1, FindingCode::UnclosedEnvelope, message));
    }
    if let Some(line) = scan.first_crlf_line {
        let message = "envelope uses CRLF line endings".to_owned();
        findings.push(Finding::at(line, FindingCode::Crlf, message));
    }
    // A missing final newline is only the envelope's to report: a body owns
    // its last bytes, quirks included.
    if scan.envelope.close_fence.is_some() && body.is_empty() && !text.ends_with('\n') {
        let message = "no newline at end of file".to_owned();
        findings.push(Finding::at(
            lines.len(),
            FindingCode::NoFinalNewline,
            message,
        ));
    }

    RecordFile {
        bom: false,
        envelope: Some(scan.envelope),
        body,
        findings,
    }
}

fn file_without_envelope(text: &str) -> RecordFile {
    let message = "file does not start with a `---` fence".to_owned();
    RecordFile {
        bom: false,
        envelope: None,
        body: text.to_owned(),
        findings: vec![Finding::at(1, FindingCode::NoEnvelope, message)],
    }
}

/// Everything the envelope scan learns line by line.
struct EnvelopeScan {
    envelope: Envelope,
    /// Byte offset where the body starts; `text.len()` when nothing follows.
    body_start: usize,
    first_crlf_line: Option<usize>,
    findings: Vec<Finding>,
}

fn scan_envelope(open_fence: &str, field_rows: &[&str]) -> EnvelopeScan {
    let mut lines = Vec::new();
    let mut close_fence = None;
    let mut findings = Vec::new();
    let (_, open_fence_crlf) = line_content(open_fence);
    let mut first_crlf_line = open_fence_crlf.then_some(1);
    let mut body_start = open_fence.len();

    // File line 1 is the open fence; these rows start at line 2.
    for (index, raw) in field_rows.iter().enumerate() {
        let line = index + 2;
        body_start += raw.len();
        let (content, crlf) = line_content(raw);
        if crlf {
            first_crlf_line.get_or_insert(line);
        }
        if content == FENCE {
            close_fence = Some((*raw).to_owned());
            break;
        }
        if let Some((key, value)) = parse_field_line(content) {
            lines.push(EnvelopeLine::Field(FieldLine {
                key,
                value,
                raw: (*raw).to_owned(),
                line: Some(line),
            }));
        } else {
            let message = "expected a `key: value` field or a `---` fence".to_owned();
            findings.push(Finding::at(line, FindingCode::BadEnvelopeLine, message));
            lines.push(EnvelopeLine::Raw((*raw).to_owned()));
        }
    }

    EnvelopeScan {
        envelope: Envelope {
            open_fence: open_fence.to_owned(),
            lines,
            close_fence,
        },
        body_start,
        first_crlf_line,
        findings,
    }
}

/// The line without its terminator, and whether that terminator was CRLF.
fn line_content(raw: &str) -> (&str, bool) {
    match raw.strip_suffix('\n') {
        Some(content) => match content.strip_suffix('\r') {
            Some(content) => (content, true),
            None => (content, false),
        },
        None => (raw, false),
    }
}

/// Strict on the key, lenient on the separator: any run of spaces or tabs
/// after `:` is accepted, and the value is trimmed of trailing whitespace.
pub(crate) fn parse_field_line(content: &str) -> Option<(String, String)> {
    let (key, rest) = content.split_once(':')?;
    let mut chars = key.chars();
    if !chars.next()?.is_ascii_lowercase() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return None;
    }
    let value = rest.trim_start_matches([' ', '\t']).trim_end();
    Some((key.to_owned(), value.to_owned()))
}

/// `archive/tasks/task.x.md` → (`Some("archive/tasks")`, `task.x.md`): the
/// whole parent path, since a home is a path and not a directory name. A
/// bare filename has no parents to claim.
fn split_path(path: &str) -> (Option<&str>, &str) {
    match path.rsplit_once('/') {
        Some((parents, filename)) => (Some(parents), filename),
        None => (None, path),
    }
}

/// The semantic pass over parsed field lines: unknown and duplicate keys,
/// value forms, required fields, and id/type coherence.
fn check_fields(envelope: &Envelope, findings: &mut Vec<Finding>) {
    let mut seen: Vec<(&str, Option<usize>)> = Vec::new();
    for field in envelope.fields() {
        let Some(spec) = field_spec(&field.key) else {
            let message = format!("unknown field `{}`", field.key);
            findings.push(Finding::located(
                field.line,
                FindingCode::UnknownField,
                message,
            ));
            continue;
        };
        let earlier = seen.iter().find(|(key, _)| *key == spec.key);
        if let Some((_, first_line)) = earlier
            && !spec.repeatable
        {
            let message = match first_line {
                Some(first_line) => {
                    format!("field `{}` is already set on line {first_line}", field.key)
                }
                None => format!("field `{}` is already set", field.key),
            };
            findings.push(Finding::located(
                field.line,
                FindingCode::DuplicateField,
                message,
            ));
            continue;
        }
        // A repeatable key records only its first line: `seen` is scanned
        // once per field, and one entry per line would make that scan
        // quadratic in the record's repeated fields.
        if earlier.is_none() {
            seen.push((spec.key, field.line));
        }
        findings.extend(value_finding(spec, field));
    }

    for spec in FIELD_TABLE {
        if spec.required && !seen.iter().any(|(key, _)| *key == spec.key) {
            let message = format!("missing required field `{}`", spec.key);
            findings.push(Finding::for_file(FindingCode::MissingField, message));
        }
    }

    check_id_type_coherence(envelope, findings);
}

/// The id names its type (`task.x`) and the envelope names it again in
/// `type`; the two claiming different types is as incoherent as a wrong
/// directory, and would otherwise pass every per-field check.
fn check_id_type_coherence(envelope: &Envelope, findings: &mut Vec<Finding>) {
    let Some(id) = envelope.first("id") else {
        return;
    };
    let Some(type_field) = envelope.first("type") else {
        return;
    };
    if id_error(&id.value).is_some() || type_word_error(&type_field.value).is_some() {
        return;
    }
    let id_type = id.value.split_once('.').map(|(type_word, _)| type_word);
    if id_type != Some(type_field.value.as_str()) {
        let message = format!(
            "id type `{}` does not match `type: {}`",
            id_type.unwrap_or_default(),
            type_field.value
        );
        findings.push(Finding::located(id.line, FindingCode::BadId, message));
    }
}

/// One finding shape for every form: `key: reason`, at the field's line.
fn value_finding(spec: &FieldSpec, field: &FieldLine) -> Option<Finding> {
    let value = field.value.as_str();
    let (code, reason) = match spec.form {
        Form::Text => return None,
        Form::NonEmptyText => (FindingCode::BadValue, non_empty_error(value)?),
        Form::LowerWord => (FindingCode::BadValue, lower_word_error(value)?),
        Form::TypeWord => (FindingCode::BadValue, type_word_error(value)?),
        Form::TagList => (FindingCode::BadValue, tag_list_error(value)?),
        Form::Link => (FindingCode::BadValue, link_error(value)?),
        Form::Priority => (FindingCode::BadValue, priority_error(value)?),
        Form::Id => (FindingCode::BadId, id_error(value)?),
        Form::Date => (FindingCode::BadDate, date_error(value)?),
    };
    let message = format!("{}: {reason}", spec.key);
    Some(Finding::located(field.line, code, message))
}

fn non_empty_error(value: &str) -> Option<String> {
    value.is_empty().then(|| "must not be empty".to_owned())
}

fn priority_error(value: &str) -> Option<String> {
    let in_range = matches!(value, "0" | "1" | "2" | "3" | "4");
    (!in_range).then(|| format!("`{value}` is not an integer 0 to 4"))
}

pub(crate) fn is_lower_word(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn lower_word_error(value: &str) -> Option<String> {
    (!is_lower_word(value)).then(|| format!("`{value}` is not one lowercase word"))
}

fn type_word_error(value: &str) -> Option<String> {
    (!TYPE_WORDS.contains(&value))
        .then(|| format!("`{value}` is not one of {}", TYPE_WORDS.join(", ")))
}

fn tag_list_error(value: &str) -> Option<String> {
    let bad_token = value
        .split(',')
        .map(str::trim)
        .find(|token| !is_token(token));
    bad_token.map(|token| format!("`{token}` is not a `[a-z0-9-]+` tag"))
}

fn link_error(value: &str) -> Option<String> {
    let well_formed = split_link(value).is_some_and(|(kind, target)| is_link(kind, target));
    (!well_formed).then(|| format!("`{value}` is not `<kind> <target>`"))
}

/// A link line split into its kind and its target: a token, then the rest
/// of the line.
pub(crate) fn split_link(link: &str) -> Option<(&str, &str)> {
    let (kind, target) = link.split_once(char::is_whitespace)?;
    Some((kind, target.trim()))
}

/// Whether a split link is well formed. The halves are judged apart from
/// the line they came from, so a caller holding them already — a draft's
/// `--link` — asks the same question the field table asks.
pub(crate) fn is_link(kind: &str, target: &str) -> bool {
    is_token(kind) && !target.trim().is_empty()
}

pub(crate) fn date_error(value: &str) -> Option<String> {
    let valid = date::is_date(value) || date::is_timestamp(value);
    (!valid).then(|| format!("`{value}` is not `YYYY-MM-DD` or an RFC 3339 timestamp"))
}

/// How long an id may be, in bytes.
pub(crate) const ID_CAP: usize = 64;

/// The id grammar: `<type>.<slug>`, ASCII, at most [`ID_CAP`] bytes. The
/// first dot splits: slugs contain no dots.
pub(crate) fn id_error(value: &str) -> Option<String> {
    if !value.is_ascii() {
        return Some(format!("`{value}` contains non-ASCII characters"));
    }
    if value.len() > ID_CAP {
        return Some(format!("`{value}` is longer than {ID_CAP} bytes"));
    }
    let Some((type_word, slug)) = value.split_once('.') else {
        return Some(format!("`{value}` is not `<type>.<slug>`"));
    };
    if !TYPE_WORDS.contains(&type_word) {
        return Some(format!("`{type_word}` is not a record type"));
    }
    if !is_slug(slug) {
        return Some(format!(
            "`{slug}` does not match `[a-z0-9]([a-z0-9-]*[a-z0-9])?`"
        ));
    }
    None
}

pub(crate) fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// A single character is a legal slug: it is both ends at once.
fn is_slug(slug: &str) -> bool {
    let bytes = slug.as_bytes();
    let alphanumeric = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    !bytes.is_empty()
        && alphanumeric(bytes[0])
        && alphanumeric(bytes[bytes.len() - 1])
        && bytes.iter().all(|&byte| alphanumeric(byte) || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQUIRED: [&str; 5] = [
        "id: task.demo-record",
        "type: task",
        "state: open",
        "title: A demo record",
        "created: 2026-08-24",
    ];

    fn record(field_lines: &[&str], body: &str) -> String {
        let mut text = String::from("---\n");
        for line in field_lines {
            text.push_str(line);
            text.push('\n');
        }
        text.push_str("---\n");
        text.push_str(body);
        text
    }

    /// The required lines with one of them replaced, so a test shows only its
    /// own override.
    fn required_with(key: &str, replacement: &'static str) -> Vec<&'static str> {
        REQUIRED
            .iter()
            .map(|line| {
                if line.split(':').next() == Some(key) {
                    replacement
                } else {
                    line
                }
            })
            .collect()
    }

    fn codes(file: &RecordFile) -> Vec<(Option<usize>, FindingCode)> {
        file.findings()
            .iter()
            .map(|finding| (finding.line, finding.code))
            .collect()
    }

    /// A key may repeat, and `field` answers with the first line that
    /// carries it — the envelope reads top down, and a later line is a
    /// second value, never a correction of the one above.
    #[test]
    fn a_repeated_key_reads_as_its_first_line() {
        let file =
            RecordFile::parse("---\nid: task.demo\nlink: sha aaa\nlink: sha bbb\n---\nbody\n");
        assert_eq!(file.field("link"), Some("sha aaa"));
        assert_eq!(
            file.field_values("link").collect::<Vec<_>>(),
            vec!["sha aaa", "sha bbb"]
        );
    }

    #[test]
    fn normalize_orders_fields_canonically_and_fixes_separators() {
        let text = record(
            &[
                "created:2026-08-24",
                "title: A demo record  ",
                "custom: kept after known fields",
                "state: open",
                "type: task",
                "id: task.demo-record",
            ],
            "body\n",
        );
        let normalized = RecordFile::parse(&text).normalize();
        assert_eq!(
            normalized,
            "---\n\
             id: task.demo-record\n\
             type: task\n\
             state: open\n\
             title: A demo record\n\
             created: 2026-08-24\n\
             custom: kept after known fields\n\
             ---\n\
             body\n"
        );
    }

    #[test]
    fn an_empty_envelope_names_every_missing_required_field() {
        let file = RecordFile::parse("---\n---\n");
        assert!(file.has_errors());
        assert_eq!(
            codes(&file),
            vec![(None, FindingCode::MissingField); 5],
            "one finding per required field"
        );
        for key in ["id", "type", "state", "title", "created"] {
            assert!(
                file.findings()
                    .iter()
                    .any(|finding| finding.message.contains(&format!("`{key}`"))),
                "no finding names `{key}`: {:?}",
                file.findings()
            );
        }
    }

    #[test]
    fn placement_does_not_re_report_an_id_that_already_failed_its_own_check() {
        let file = RecordFile::parse(&record(&required_with("id", "id: task.X"), ""));
        assert_eq!(file.placement_findings("tasks/task.other.md"), vec![]);
    }

    #[test]
    fn set_field_replaces_only_its_own_line_and_keeps_every_other_quirk() {
        let text = record(
            &[
                "id:task.demo-record",
                "type:  task",
                "state: open",
                "custom: kept verbatim   ",
                "title: A demo record",
                "created: 2026-08-24",
            ],
            "body\n",
        );
        let mut file = RecordFile::parse(&text);
        assert!(file.set_field("state", "active"));
        assert_eq!(
            file.render(),
            record(
                &[
                    "id:task.demo-record",
                    "type:  task",
                    "state: active",
                    "custom: kept verbatim   ",
                    "title: A demo record",
                    "created: 2026-08-24",
                ],
                "body\n",
            )
        );
    }

    #[test]
    fn set_field_inserts_a_new_field_at_its_canonical_position() {
        let mut file = RecordFile::parse(&record(&REQUIRED, "body\n"));
        file.set_field("hold", "waiting for the 1.99 release");
        assert_eq!(
            file.render(),
            record(
                &[
                    "id: task.demo-record",
                    "type: task",
                    "state: open",
                    "title: A demo record",
                    "hold: waiting for the 1.99 release",
                    "created: 2026-08-24",
                ],
                "body\n",
            )
        );
    }

    #[test]
    fn set_field_with_the_same_bytes_reports_no_change() {
        let text = record(&REQUIRED, "body\n");
        let mut file = RecordFile::parse(&text);
        assert!(!file.set_field("state", "open"));
        assert_eq!(file.render(), text);
    }

    #[test]
    fn splicing_into_a_crlf_file_writes_the_touched_line_lf_and_keeps_the_rest() {
        let text = record(&REQUIRED, "body\n").replace('\n', "\r\n");
        let mut file = RecordFile::parse(&text);
        file.set_field("state", "active");
        assert_eq!(
            file.render(),
            text.replace("state: open\r\n", "state: active\n")
        );
    }

    #[test]
    fn append_field_adds_a_line_after_the_keys_last_occurrence() {
        let mut lines = REQUIRED.to_vec();
        lines.push("link: doc a.md");
        lines.push("link: doc b.md");
        let mut file = RecordFile::parse(&record(&lines, ""));
        file.append_field("link", "pr https://example.com/1");
        assert_eq!(
            file.field_values("link").collect::<Vec<_>>(),
            vec!["doc a.md", "doc b.md", "pr https://example.com/1"]
        );
    }

    #[test]
    fn remove_field_removes_every_line_of_the_key_and_nothing_else() {
        let mut lines = REQUIRED.to_vec();
        lines.push("hold: a reason");
        lines.push("hold-until: 2026-09-01");
        let mut file = RecordFile::parse(&record(&lines, "body\n"));
        assert!(file.remove_field("hold"));
        assert!(file.remove_field("hold-until"));
        assert!(!file.remove_field("hold"));
        assert_eq!(file.render(), record(&REQUIRED, "body\n"));
    }

    #[test]
    fn append_body_starts_the_body_even_when_the_envelope_lacks_its_newline() {
        let text = record(&REQUIRED, "");
        let mut file = RecordFile::parse(text.strip_suffix('\n').unwrap());
        file.append_body("- a log line");
        assert_eq!(file.render(), format!("{text}- a log line\n"));
    }

    #[test]
    fn append_body_supplies_the_newline_a_bodys_last_line_lacks() {
        let mut file = RecordFile::parse(&record(&REQUIRED, "no newline at the end"));
        file.append_body("- a log line");
        assert_eq!(file.body(), "no newline at the end\n- a log line\n");
    }
}
