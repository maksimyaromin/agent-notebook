//! Keep follow-up commands in the same notebook and explicit session.

use anb::cli::Cli;
use anb_core::encode::shell_word;
use clap::Parser;
use serde_json::Value;

pub struct Actions {
    scope: String,
    session: Option<String>,
}

impl Actions {
    pub fn new(cli: &Cli) -> Self {
        let scope = if cli.global {
            " --global".to_owned()
        } else if cli.personal {
            " --personal".to_owned()
        } else {
            cli.notebook.as_ref().map_or_else(String::new, |path| {
                format!(" --notebook {}", shell_word(&path.to_string_lossy()))
            })
        };
        Self {
            scope,
            session: cli.session.clone(),
        }
    }

    pub fn qualify(&self, document: &mut Value) {
        match document {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if matches!(key.as_str(), "more" | "read" | "team" | "try" | "repair") {
                        self.commands(value);
                    } else {
                        self.qualify(value);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    self.qualify(item);
                }
            }
            _ => {}
        }
    }

    fn commands(&self, value: &mut Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.commands(item);
                }
            }
            Value::String(command) => {
                let Some(rest) = command.strip_prefix("anb ") else {
                    return;
                };
                let Some(words) = shlex::split(command) else {
                    return;
                };
                let Ok(parsed) = Cli::try_parse_from(words) else {
                    return;
                };
                let scope = if parsed.global || parsed.personal || parsed.notebook.is_some() {
                    ""
                } else {
                    &self.scope
                };
                let session = self
                    .session
                    .as_ref()
                    .filter(|_| parsed.session.is_none())
                    .map_or_else(String::new, |session| {
                        format!(" --session {}", shell_word(session))
                    });
                *command = format!("anb{scope}{session} {rest}");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn followups_keep_the_explicit_notebook_and_session() {
        let cli = Cli::parse_from([
            "anb",
            "--notebook",
            "a notebook",
            "--session",
            "work one",
            "list",
        ]);
        let mut document = json!({"more": "anb list --all", "try": ["anb show task.one"], "body": {"head": "anb list --all"}});
        Actions::new(&cli).qualify(&mut document);
        assert_eq!(
            document["more"],
            "anb --notebook 'a notebook' --session 'work one' list --all"
        );
        assert_eq!(
            document["try"][0],
            "anb --notebook 'a notebook' --session 'work one' show task.one"
        );
        assert_eq!(document["body"]["head"], "anb list --all");
    }

    #[test]
    fn explicit_audience_and_recovery_session_are_not_overridden() {
        let cli = Cli::parse_from(["anb", "--personal", "--session", "current", "recall"]);
        let mut document = json!({"memories": [{"read": "anb --global show note.rule"}], "try": ["anb start --session other"]});
        Actions::new(&cli).qualify(&mut document);
        assert_eq!(
            document["memories"][0]["read"],
            "anb --session current --global show note.rule"
        );
        assert_eq!(document["try"][0], "anb --personal start --session other");
    }

    #[test]
    fn flag_words_inside_a_title_are_data_not_scope() {
        let cli = Cli::parse_from(["anb", "--global", "list"]);
        let mut document = json!({"try": ["anb add note '--personal --notebook examples'"]});
        Actions::new(&cli).qualify(&mut document);
        assert_eq!(
            document["try"][0],
            "anb --global add note '--personal --notebook examples'"
        );
    }
}
