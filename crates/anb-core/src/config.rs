//! The notebook config: flat `key: value` lines in the envelope line
//! grammar, no fences.
//!
//! Every key has a default and a bad line falls back to it: config trouble
//! surfaces through `check` as findings, never as a refused Status — the
//! session-start hook must fail soft. Absence of the file is legal.

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

// Every key the config carries, all integers: `format` names the format
// version the notebook is written in, `budget` the Status ceiling (`0` = no
// ceiling), the `debt-*` keys the clock thresholds in days. Each named
// constant is both a table row and the extraction handle, so a key cannot
// drift from its default — and `format`'s default is the version this build
// reads, which is what makes a disagreement a finding rather than a guess.
const FORMAT: ConfigKey = config_key("format", 1);
const BUDGET: ConfigKey = config_key("budget", Budget::DEFAULT_TOKENS);
const TASK_STALE: ConfigKey = config_key("debt-task-stale", 7);
const QUESTION_AGE: ConfigKey = config_key("debt-question-age", 14);
const QUESTION_AGE_TASK_BORN: ConfigKey = config_key("debt-question-age-task-born", 7);
const HOLD_QUIET: ConfigKey = config_key("debt-hold-quiet", 14);
const REVIEW_WAIT: ConfigKey = config_key("debt-review-wait", 7);

const CONFIG_KEYS: &[ConfigKey] = &[
    FORMAT,
    BUDGET,
    TASK_STALE,
    QUESTION_AGE,
    QUESTION_AGE_TASK_BORN,
    HOLD_QUIET,
    REVIEW_WAIT,
];

/// The parsed config: every value resolved, the file's findings kept for
/// `check`.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    budget: Budget,
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
        let (values, findings) = read_values(text);
        let value = |spec: ConfigKey| {
            values
                .iter()
                .find(|(found, _)| *found == spec.key)
                .map_or(spec.default, |(_, value)| *value)
        };
        Config {
            budget: Budget::from_ceiling(value(BUDGET)),
            debt: DebtThresholds {
                task_stale: value(TASK_STALE),
                question_age: value(QUESTION_AGE),
                question_age_task_born: value(QUESTION_AGE_TASK_BORN),
                hold_quiet: value(HOLD_QUIET),
                review_wait: value(REVIEW_WAIT),
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

    #[must_use]
    pub fn debt_thresholds(&self) -> DebtThresholds {
        self.debt
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }
}

fn read_values(text: &str) -> (Vec<(String, u32)>, Vec<Finding>) {
    let mut values: Vec<(String, u32)> = Vec::new();
    let mut findings = Vec::new();
    for (index, content) in text.lines().enumerate() {
        let line = index + 1;
        if content.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = grammar::parse_field_line(content) else {
            let message = "expected a `key: value` line".to_owned();
            findings.push(Finding::at(line, FindingCode::BadEnvelopeLine, message));
            continue;
        };
        if !CONFIG_KEYS.iter().any(|spec| spec.key == key) {
            let message = format!("unknown config key `{key}`");
            findings.push(Finding::at(line, FindingCode::UnknownField, message));
            continue;
        }
        if values.iter().any(|(found, _)| *found == key) {
            let message = format!("key `{key}` is already set");
            findings.push(Finding::at(line, FindingCode::DuplicateField, message));
            continue;
        }
        match parse_integer(&value) {
            Ok(parsed) if key == FORMAT.key && parsed != FORMAT.default => {
                let message = format!(
                    "format: `{parsed}` is not the format this anb reads (`{}`) — the notebook was written by another version",
                    FORMAT.default
                );
                findings.push(Finding::at(line, FindingCode::BadValue, message));
            }
            Ok(parsed) => values.push((key, parsed)),
            Err(why) => {
                let message = format!("{key}: `{value}` {why}");
                findings.push(Finding::at(line, FindingCode::BadValue, message));
            }
        }
    }
    (values, findings)
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
