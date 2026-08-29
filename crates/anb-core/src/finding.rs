//! Check findings — the named outcomes of verification.
//!
//! A parse either accepts a file (possibly with warnings) or rejects it with
//! findings naming the line and reason; silent loss is not an outcome Check
//! permits. The codes are a closed, documented set; each layer adds the
//! codes it can detect.

/// The weight of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The record is excluded from mutation and derived queries, listed as
    /// invalid, and its bytes are never rewritten while invalid.
    Error,
    /// The record stays fully usable; `check` reports it.
    Warning,
}

impl Severity {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// The stable kebab-case finding codes: the grammar layer's and the
/// record model's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindingCode {
    NoEnvelope,
    UnclosedEnvelope,
    BadEnvelopeLine,
    DuplicateField,
    UnknownField,
    MissingField,
    BadValue,
    BadDate,
    BadId,
    IdFilenameMismatch,
    TypeDirMismatch,
    ArchivedLiveRecord,
    UnarchivedSettledRecord,
    OrphanField,
    BrokenRouting,
    DanglingRef,
    DepCycle,
    DuplicateId,
    BrokenSupersession,
    NotUtf8,
    Crlf,
    Bom,
    NoFinalNewline,
}

impl FindingCode {
    #[must_use]
    pub fn severity(self) -> Severity {
        match self {
            FindingCode::NoEnvelope
            | FindingCode::UnclosedEnvelope
            | FindingCode::BadEnvelopeLine
            | FindingCode::DuplicateField
            | FindingCode::MissingField
            | FindingCode::BadValue
            | FindingCode::BadDate
            | FindingCode::BadId
            | FindingCode::IdFilenameMismatch
            | FindingCode::TypeDirMismatch
            | FindingCode::ArchivedLiveRecord
            | FindingCode::BrokenRouting
            | FindingCode::DanglingRef
            | FindingCode::DepCycle
            | FindingCode::DuplicateId
            | FindingCode::BrokenSupersession
            | FindingCode::NotUtf8 => Severity::Error,
            FindingCode::UnknownField
            | FindingCode::UnarchivedSettledRecord
            | FindingCode::OrphanField
            | FindingCode::Crlf
            | FindingCode::Bom
            | FindingCode::NoFinalNewline => Severity::Warning,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FindingCode::NoEnvelope => "no-envelope",
            FindingCode::UnclosedEnvelope => "unclosed-envelope",
            FindingCode::BadEnvelopeLine => "bad-envelope-line",
            FindingCode::DuplicateField => "duplicate-field",
            FindingCode::UnknownField => "unknown-field",
            FindingCode::MissingField => "missing-field",
            FindingCode::BadValue => "bad-value",
            FindingCode::BadDate => "bad-date",
            FindingCode::BadId => "bad-id",
            FindingCode::IdFilenameMismatch => "id-filename-mismatch",
            FindingCode::TypeDirMismatch => "type-dir-mismatch",
            FindingCode::ArchivedLiveRecord => "archived-live-record",
            FindingCode::UnarchivedSettledRecord => "unarchived-settled-record",
            FindingCode::OrphanField => "orphan-field",
            FindingCode::BrokenRouting => "broken-routing",
            FindingCode::DanglingRef => "dangling-ref",
            FindingCode::DepCycle => "dep-cycle",
            FindingCode::DuplicateId => "duplicate-id",
            FindingCode::BrokenSupersession => "broken-supersession",
            FindingCode::NotUtf8 => "not-utf8",
            FindingCode::Crlf => "crlf",
            FindingCode::Bom => "bom",
            FindingCode::NoFinalNewline => "no-final-newline",
        }
    }
}

impl std::fmt::Display for FindingCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verification outcome: the code, where it fired, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: FindingCode,
    /// 1-based line in the file; `None` for findings about the whole file.
    pub line: Option<usize>,
    pub message: String,
}

impl Finding {
    #[must_use]
    pub fn at(line: usize, code: FindingCode, message: String) -> Self {
        Self {
            code,
            line: Some(line),
            message,
        }
    }

    /// A finding about the file as a whole, with no line to point at.
    #[must_use]
    pub fn for_file(code: FindingCode, message: String) -> Self {
        Self {
            code,
            line: None,
            message,
        }
    }

    /// A finding wherever its field sits — a known line, or none on a field
    /// spliced in by a mutation.
    #[must_use]
    pub fn located(line: Option<usize>, code: FindingCode, message: String) -> Self {
        Self {
            code,
            line,
            message,
        }
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}: {}", self.code, self.message),
            None => write!(f, "{}: {}", self.code, self.message),
        }
    }
}
