//! The write path up to the file: every guard that judges a request
//! before a record is read, the id a draft mints, and the bytes it
//! becomes. The write itself is the verb's, in the parent.

use super::error::NotebookError;
use crate::date;
use crate::grammar::{self, RecordFile};
use crate::record::{Record, RecordType};
use crate::request::{CLEARABLE, Draft, Edit, PRIORITY, Proof};
use crate::resolve::{path_stem, record_path, type_of};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_draft(draft: &Draft) -> Result<(), NotebookError> {
    let invalid = |reason: String| Err(NotebookError::InvalidArgument { reason });

    let title = draft.title.trim();
    guard_single_line("title", title)?;
    if title.is_empty() {
        return invalid("title: must not be empty".to_owned());
    }
    for (field, value) in [
        ("by", &draft.by),
        ("via", &draft.via),
        ("kind", &draft.kind),
    ] {
        if let Some(value) = value {
            guard_single_line(field, value)?;
        }
    }

    if let Some(kind) = &draft.kind {
        match draft.record_type.kinds() {
            None => {
                return invalid(format!(
                    "kind: a {} carries no kind",
                    draft.record_type.word()
                ));
            }
            Some(kinds) if !kinds.contains(&kind.as_str()) => {
                return invalid(format!(
                    "kind: `{kind}` is not one of {} for a {}",
                    kinds.join(", "),
                    draft.record_type.word()
                ));
            }
            Some(_) => {}
        }
    }

    if let Some(priority) = draft.priority {
        if draft.record_type != RecordType::Task {
            return invalid("priority: applies only to a task".to_owned());
        }
        if priority > 4 {
            return invalid(format!("priority: {priority} is not 0–4"));
        }
    }

    if draft.supersedes.is_some()
        && !matches!(draft.record_type, RecordType::Decision | RecordType::Note)
    {
        return invalid(format!(
            "supersedes: a {} closes through its own lifecycle, not supersession",
            draft.record_type.word()
        ));
    }

    for tag in &draft.tags {
        if !grammar::is_token(tag) {
            return invalid(format!("tags: `{tag}` is not a `[a-z0-9-]+` tag"));
        }
    }
    for link in &draft.links {
        guard_single_line("link", &link.target)?;
        if !grammar::is_link(&link.kind, &link.target) {
            let given = format!("{} {}", link.kind, link.target);
            return invalid(format!(
                "link: `{}` is not `<kind> <target>`",
                given.trim_end()
            ));
        }
    }
    Ok(())
}

/// Judge an [`Edit`] whole and hand back the envelope keys `--clear`
/// named, so the splice never has to re-read the caller's spelling of
/// them: a field is matched against [`CLEARABLE`] exactly once, here.
///
/// # Errors
/// [`NotebookError::InvalidArgument`] on any malformed or contradictory
/// part of the request.
pub(super) fn validate_edit(
    record_type: RecordType,
    edit: &Edit,
) -> Result<Vec<&'static str>, NotebookError> {
    let invalid = |reason: String| Err(NotebookError::InvalidArgument { reason });

    if edit.changes_nothing() {
        return invalid(
            "edit: nothing to change — pass --title, --body, --tag, --untag, --from, --priority, --review-by, or --clear"
                .to_owned(),
        );
    }
    if let Some(title) = &edit.title {
        let title = title.trim();
        guard_single_line("title", title)?;
        if title.is_empty() {
            return invalid("title: must not be empty".to_owned());
        }
    }
    for tag in edit.add_tags.iter().chain(&edit.remove_tags) {
        if !grammar::is_token(tag) {
            return invalid(format!("tags: `{tag}` is not a `[a-z0-9-]+` tag"));
        }
    }
    if let Some(priority) = edit.priority {
        if record_type != RecordType::Task {
            return invalid("priority: applies only to a task".to_owned());
        }
        if priority > 4 {
            return invalid(format!("priority: {priority} is not 0–4"));
        }
    }
    if let Some(date) = &edit.review_by
        && let Some(why) = grammar::date_error(date)
    {
        return invalid(format!("review-by: {why}"));
    }
    let mut cleared = Vec::new();
    for field in &edit.clear {
        let Some(key) = CLEARABLE.into_iter().find(|key| *key == field.as_str()) else {
            return invalid(format!(
                "clear: `{field}` is not an erasable field — {}",
                CLEARABLE.join(", ")
            ));
        };
        if writes(edit, key) {
            return invalid(format!("clear: `{key}` is both written and cleared"));
        }
        cleared.push(key);
    }
    Ok(cleared)
}

/// Whether the same call also writes `key`: the one contradiction a clear
/// can carry, asked of the list that will do the writing.
fn writes(edit: &Edit, key: &str) -> bool {
    edited_fields(edit, &[])
        .iter()
        .any(|(written, value)| *written == key && value.is_some())
}

/// A proof is a link value: one non-empty line, or an explicit waiver.
pub(super) fn guard_proof(proof: &Proof) -> Result<(), NotebookError> {
    let (Proof::Pr(target) | Proof::Sha(target) | Proof::Report(target) | Proof::Note(target)) =
        proof
    else {
        return Ok(());
    };
    guard_single_line("proof", target)?;
    if target.trim().is_empty() {
        return Err(NotebookError::InvalidArgument {
            reason: "proof: the target must not be empty".to_owned(),
        });
    }
    Ok(())
}
/// [`guard_today`] plus the day number the clocks subtract from.
pub(super) fn guarded_day(today: &str) -> Result<i64, NotebookError> {
    guard_today(today)?;
    date::day_number(today).ok_or_else(|| NotebookError::InvalidArgument {
        reason: format!("today: `{today}` is not a date"),
    })
}
pub(super) fn guard_today(today: &str) -> Result<(), NotebookError> {
    match grammar::date_error(today) {
        None => Ok(()),
        Some(why) => Err(NotebookError::InvalidArgument {
            reason: format!("today: {why}"),
        }),
    }
}
pub(super) fn guard_single_line(field: &str, value: &str) -> Result<(), NotebookError> {
    if value.contains('\n') {
        return Err(NotebookError::InvalidArgument {
            reason: format!("{field}: must be one line"),
        });
    }
    Ok(())
}
pub(super) fn parsed_type(id: &str) -> Result<RecordType, NotebookError> {
    match grammar::id_error(id) {
        Some(why) => Err(NotebookError::InvalidArgument {
            reason: format!("id: {why}"),
        }),
        None => Ok(type_of(id).expect("a valid id names a type")),
    }
}

/// The draft's id: the caller's, validated and free, or one minted from
/// the title — retried with a two-character suffix on collision, since
/// ids are never reused.
pub(super) fn resolve_draft_id(
    draft: &Draft,
    claims: &BTreeMap<String, String>,
) -> Result<String, NotebookError> {
    if let Some(id) = &draft.id {
        if let Some(why) = grammar::id_error(id) {
            return Err(NotebookError::InvalidArgument {
                reason: format!("id: {why}"),
            });
        }
        let id_type = parsed_type(id)?;
        if id_type != draft.record_type {
            return Err(NotebookError::InvalidArgument {
                reason: format!(
                    "id: `{id}` names a {}, the draft is a {}",
                    id_type.word(),
                    draft.record_type.word()
                ),
            });
        }
        if let Some(holder) = claims.get(id.as_str()) {
            return Err(NotebookError::DuplicateId {
                id: id.clone(),
                holder: holder.clone(),
            });
        }
        return Ok(id.clone());
    }

    let slug = slugify(&draft.title);
    if slug.is_empty() {
        return Err(NotebookError::InvalidArgument {
            reason: "title: yields an empty id — pass an explicit id".to_owned(),
        });
    }
    let base = format!("{}.{slug}", draft.record_type.word());
    if !claims.contains_key(&base) {
        return Ok(base);
    }
    for attempt in 0..SUFFIX_COUNT {
        let candidate = format!("{base}-{}", base36_pair(attempt));
        if !claims.contains_key(&candidate) {
            return Ok(candidate);
        }
    }
    Err(NotebookError::InvalidArgument {
        reason: format!("id: no free id near `{base}`"),
    })
}

/// Every id already claimed, mapped to the file claiming it.
///
/// A read record claims two names: the one on its file, and the one its
/// bytes declare — write-side uniqueness cannot trust the convention whose
/// violation is the very finding it guards against. A filed record claims
/// the name on its file alone, which its listing carries: opening the
/// archive on every create would make minting an id cost the whole of
/// history, and a file whose `id` disagrees with its name is an error
/// `check` names wherever it sits.
pub(super) fn id_claims(
    records: &[Record],
    archived: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let mut claims = BTreeMap::new();
    for record in records {
        claims
            .entry(path_stem(record.path()).to_owned())
            .or_insert_with(|| record.path().to_owned());
        if let Some(id) = record.id() {
            claims
                .entry(id.to_owned())
                .or_insert_with(|| record.path().to_owned());
        }
    }
    for id in archived {
        let Ok(record_type) = parsed_type(id) else {
            continue;
        };
        claims
            .entry(id.clone())
            .or_insert_with(|| record_path(id, record_type, true));
    }
    claims
}

/// What an ingested report is called, so a reader scanning `list` sees
/// whose report it is and never mistakes it for the Task itself.
pub(super) fn report_note_title(task_title: &str) -> String {
    format!("Report: {task_title}")
}

/// Splice every requested correction into the file, answering the keys
/// that actually moved. `cleared` is [`validate_edit`]'s reading of
/// `--clear`.
pub(super) fn spliced(
    file: &mut RecordFile,
    edit: &Edit,
    cleared: &[&'static str],
) -> Vec<&'static str> {
    let mut changed = Vec::new();
    for (key, value) in edited_fields(edit, cleared) {
        let touched = match value {
            Some(value) => file.set_field(key, &value),
            None => file.remove_field(key),
        };
        if touched {
            changed.push(key);
        }
    }
    if retagged(file, edit) {
        changed.push("tags");
    }
    if let Some(body) = &edit.body {
        let body = edited_body(body);
        if body != file.body() {
            file.set_body(&body);
            changed.push("body");
        }
    }
    changed
}

/// The envelope lines this edit means to write, in splice order: `Some`
/// sets the line, `None` erases it. Written and cleared fields share one
/// list so that what an edit changes and what it may change over cannot
/// drift apart.
fn edited_fields(edit: &Edit, cleared: &[&'static str]) -> Vec<(&'static str, Option<String>)> {
    let mut fields: Vec<(&'static str, Option<String>)> = [
        (
            "title",
            edit.title.as_deref().map(str::trim).map(str::to_owned),
        ),
        ("from", edit.from.clone()),
        (PRIORITY, edit.priority.map(|priority| priority.to_string())),
        ("review-by", edit.review_by.clone()),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|value| (key, Some(value))))
    .collect();
    fields.extend(cleared.iter().map(|key| (*key, None)));
    fields
}

/// Splice the edit's tag additions and removals into the `tags` field;
/// answers whether the set changed. An added tag already present, or a
/// removed one already absent, changes nothing — the replay contract at
/// set granularity.
fn retagged(file: &mut RecordFile, edit: &Edit) -> bool {
    if edit.add_tags.is_empty() && edit.remove_tags.is_empty() {
        return false;
    }
    let mut tags: Vec<String> = file
        .field("tags")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    tags.retain(|tag| !edit.remove_tags.contains(tag));
    for tag in &edit.add_tags {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    if tags.is_empty() {
        file.remove_field("tags")
    } else {
        file.set_field("tags", &tags.join(", "))
    }
}

/// The canonical rendered body: a blank line after the fence, the content,
/// a final newline — and empty content is no body at all. `create` and
/// `edit` both write through it, so a fresh record and an edited one read
/// alike.
pub(super) fn edited_body(content: &str) -> String {
    let content = content.strip_suffix('\n').unwrap_or(content);
    if content.is_empty() {
        String::new()
    } else {
        format!("\n{content}\n")
    }
}

/// Render a draft as a canonical record file. The splice machinery places
/// every field, so the canonical order has one home: the grammar's field
/// table.
pub(super) fn render_draft(draft: &Draft, id: &str, today: &str) -> String {
    let mut file = RecordFile::parse("---\n---\n");
    file.set_field("id", id);
    file.set_field("type", draft.record_type.word());
    file.set_field("state", draft.record_type.initial_state());
    file.set_field("title", draft.title.trim());
    for (key, value) in [
        ("kind", &draft.kind),
        ("by", &draft.by),
        ("via", &draft.via),
        ("from", &draft.from),
        ("supersedes", &draft.supersedes),
    ] {
        if let Some(value) = value {
            file.set_field(key, value);
        }
    }
    if !draft.tags.is_empty() {
        file.set_field("tags", &draft.tags.join(", "));
    }
    for link in &draft.links {
        file.append_field("link", &format!("{} {}", link.kind, link.target.trim()));
    }
    if let Some(priority) = draft.priority {
        file.set_field("priority", &priority.to_string());
    }
    file.set_field("created", today);
    file.set_field("updated", today);
    file.set_body(&edited_body(&draft.body));
    file.render()
}

/// How much of a title an id carries. An id must stay recognisable at a
/// glance and must fit the id grammar's own length limit with room for a
/// collision suffix; the rest of the title is a `view` away.
const SLUG_CAP: usize = 40;

/// The slug an id takes from a title: ASCII alphanumerics lowercased, every
/// other run a single hyphen, cut to [`SLUG_CAP`] at a word boundary.
///
/// An id is read far more often than it is minted, and a mid-word cut costs
/// its reader more than the characters it saves. A first word longer than
/// the cap offers no boundary to cut at, so it is cut short.
fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    if slug.len() <= SLUG_CAP {
        return slug.to_owned();
    }
    // One past the cap, so a boundary sitting exactly on it still counts.
    let within_cap = &slug[..=SLUG_CAP];
    match within_cap.rfind('-') {
        Some(boundary) => slug[..boundary].to_owned(),
        None => slug[..SLUG_CAP].to_owned(),
    }
}

/// How many suffixed ids one slug can carry: every two-character base36
/// pair.
const SUFFIX_COUNT: usize = 36 * 36;

fn base36_pair(n: usize) -> String {
    const DIGITS: [char; 36] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    debug_assert!(n < SUFFIX_COUNT);
    format!("{}{}", DIGITS[n / 36], DIGITS[n % 36])
}
