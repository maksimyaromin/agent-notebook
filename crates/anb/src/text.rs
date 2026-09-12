//! Canonical TOON output. Command data uses the shared reply document;
//! skill printing retains its Markdown artifact format.

use crate::json;
use crate::recovery::{Recovery, Subject};
use crate::reply::{Reply, SkillReply};
use anb_core::NotebookError;

#[must_use]
pub fn render(reply: &Reply) -> String {
    match reply {
        Reply::Skill(SkillReply::Printed(skill)) => skill.clone(),
        _ => format!("{}\n", json::toon(&json::value(reply))),
    }
}

#[must_use]
pub fn render_error(error: &NotebookError, subject: &Subject) -> String {
    render_recovery(&Recovery::new(error, subject))
}

#[must_use]
pub fn render_recovery(recovery: &Recovery) -> String {
    format!("{}\n", json::toon(&json::recovery_value(recovery)))
}
