//! The notebook config: flat `key: value` lines in the envelope line
//! grammar, no fences.
//!
//! Every key has a default and a bad line falls back to it: config trouble
//! surfaces through `check` as findings, never as a refused Status — the
//! session-start hook must fail soft. Absence of the file is legal.
//!
//! The integer keys are one table; `scope` is the one word-valued key,
//! judged on its own.

use crate::debt::DebtThresholds;
use crate::finding::{Finding, FindingCode};
use crate::grammar;
use crate::status::Budget;

pub(crate) const CONFIG_PATH: &str = "config";

struct ConfigKey {
    key: &'static str,
    default: u32,
}

const fn config_key(key: &'static str, default: u32) -> ConfigKey {
    ConfigKey { key, default }
}

// The integer keys: `format` names the format version the notebook is
// written in, `budget` the Status ceiling (`0` = no ceiling), the `debt-*`
// keys the clock thresholds in days. Each named constant is both a table
// row and the extraction handle, so a key cannot drift from its default.
// `format`'s default is the version this build reads, so a disagreement is
// a finding rather than a guess.
const FORMAT: ConfigKey = config_key("format", 1);
const BUDGET: ConfigKey = config_key("budget", Budget::DEFAULT_TOKENS);
const TASK_STALE: ConfigKey = config_key("debt-task-stale", 7);
const QUESTION_AGE: ConfigKey = config_key("debt-question-age", 14);
const QUESTION_AGE_TASK_BORN: ConfigKey = config_key("debt-question-age-task-born", 7);
const HOLD_STALE: ConfigKey = config_key("debt-hold-stale", 14);
const REVIEW_STALE: ConfigKey = config_key("debt-review-stale", 7);

const CONFIG_KEYS: &[ConfigKey] = &[
    FORMAT,
    BUDGET,
    TASK_STALE,
    QUESTION_AGE,
    QUESTION_AGE_TASK_BORN,
    HOLD_STALE,
    REVIEW_STALE,
];

/// The `scope` key: whose records a read answers with when the call
/// names nobody.
const SCOPE: &str = "scope";

/// Whose records a read answers with by default: everyone's, or the
/// caller's own: the Tasks the identity holds, the records it wrote and
/// the records waiting on it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Team,
    Mine,
}

impl Scope {
    const WORDS: [(&'static str, Scope); 2] = [("team", Scope::Team), ("mine", Scope::Mine)];

    fn from_word(word: &str) -> Option<Scope> {
        Scope::WORDS
            .into_iter()
            .find(|(spelled, _)| *spelled == word)
            .map(|(_, scope)| scope)
    }

    fn words() -> String {
        Scope::WORDS.map(|(word, _)| word).join(", ")
    }
}

/// The parsed config: every value resolved, the file's findings kept for
/// `check`.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    budget: Budget,
    scope: Scope,
    debt: DebtThresholds,
    findings: Vec<Finding>,
}

impl Default for Config {
    fn default() -> Self {
        Config::parse("")
    }
}

impl Config {
    /// Parse config text; an unparseable or out-of-form line yields a
    /// finding and its key keeps the default.
    #[must_use]
    pub(crate) fn parse(text: &str) -> Config {
        let Values {
            integers,
            scope,
            findings,
        } = read_values(text);
        let value = |spec: ConfigKey| {
            integers
                .iter()
                .find(|(found, _)| *found == spec.key)
                .map_or(spec.default, |(_, value)| *value)
        };
        Config {
            budget: Budget::from_ceiling(value(BUDGET)),
            scope: scope.unwrap_or_default(),
            debt: DebtThresholds {
                task_stale: value(TASK_STALE),
                question_age: value(QUESTION_AGE),
                question_age_task_born: value(QUESTION_AGE_TASK_BORN),
                hold_stale: value(HOLD_STALE),
                review_stale: value(REVIEW_STALE),
            },
            findings,
        }
    }

    /// The Status ceiling: the `budget` key through the `0 = no ceiling`
    /// rule, or the default 1500.
    #[must_use]
    pub fn budget(&self) -> Budget {
        self.budget
    }

    /// Whose records a read answers with when the call names nobody: the
    /// `scope` key, or everyone's.
    #[must_use]
    pub fn scope(&self) -> Scope {
        self.scope
    }

    #[must_use]
    pub fn debt_thresholds(&self) -> DebtThresholds {
        self.debt
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}

/// What one pass over the file read: the integer keys set, the scope when
/// set, and every line's finding.
struct Values {
    integers: Vec<(String, u32)>,
    scope: Option<Scope>,
    findings: Vec<Finding>,
}

fn read_values(text: &str) -> Values {
    let mut values = Values {
        integers: Vec::new(),
        scope: None,
        findings: Vec::new(),
    };
    let mut set: Vec<String> = Vec::new();
    for (index, content) in text.lines().enumerate() {
        let line = index + 1;
        if content.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = grammar::parse_field_line(content) else {
            let message = "expected a `key: value` line".to_owned();
            values
                .findings
                .push(Finding::at(line, FindingCode::BadEnvelopeLine, message));
            continue;
        };
        let known = key == SCOPE || CONFIG_KEYS.iter().any(|spec| spec.key == key);
        if !known {
            let message = format!("unknown config key `{key}`");
            values
                .findings
                .push(Finding::at(line, FindingCode::UnknownField, message));
            continue;
        }
        if set.contains(&key) {
            let message = format!("key `{key}` is already set");
            values
                .findings
                .push(Finding::at(line, FindingCode::DuplicateField, message));
            continue;
        }
        set.push(key.clone());
        if key == SCOPE {
            values.scope = Scope::from_word(&value);
            if values.scope.is_none() {
                let message = format!("scope: `{value}` is not one of {}", Scope::words());
                values
                    .findings
                    .push(Finding::at(line, FindingCode::BadValue, message));
            }
            continue;
        }
        match parse_integer(&value) {
            Ok(parsed) => {
                if key == FORMAT.key && parsed != FORMAT.default {
                    let message = format!(
                        "format: `{parsed}` is not the format this anb reads (`{}`); the notebook was written by another version",
                        FORMAT.default
                    );
                    values
                        .findings
                        .push(Finding::at(line, FindingCode::BadValue, message));
                }
                values.integers.push((key, parsed));
            }
            Err(why) => {
                let message = format!("{key}: `{value}` {why}");
                values
                    .findings
                    .push(Finding::at(line, FindingCode::BadValue, message));
            }
        }
    }
    values
}

/// Digits only: the config is typed by key like the envelope, so `1_000`,
/// `+1`, and empty are named findings, never guesses — and a value past
/// `u32` is named as out of range, not as a non-integer.
fn parse_integer(value: &str) -> Result<u32, &'static str> {
    if value.is_empty() || value.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err("is not a non-negative integer");
    }
    value
        .parse()
        .map_err(|_| "does not fit a config integer (max 4294967295)")
}
