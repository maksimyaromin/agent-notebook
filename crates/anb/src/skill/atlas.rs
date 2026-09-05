//! The atlas skill: how an agent draws a notebook from `anb graph --json`
//! and turns the reader's comments on the page back into commands. Written
//! by hand under `.agents/skills/anb-atlas/` in this repository, where Codex
//! and Pi read it directly, and carried in the binary so `setup` can install
//! it in any project. The committed files are the source; the binary embeds
//! them at build.

/// The skill's directory name under a host's `skills/`.
pub const NAME: &str = "anb-atlas";

/// Every file of the skill, path relative to its directory.
#[must_use]
pub fn files() -> [(&'static str, &'static str); 3] {
    [
        (
            "SKILL.md",
            include_str!("../../../../.agents/skills/anb-atlas/SKILL.md"),
        ),
        (
            "references/drawing.md",
            include_str!("../../../../.agents/skills/anb-atlas/references/drawing.md"),
        ),
        (
            "references/intent-loop.md",
            include_str!("../../../../.agents/skills/anb-atlas/references/intent-loop.md"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_file_carries_the_mark_setup_rewrites_by() {
        for (file, text) in files() {
            assert!(super::super::is_managed(text), "{file} lacks the mark");
        }
    }

    #[test]
    fn the_skill_names_its_references_where_they_lie() {
        let (_, skill) = files()[0];
        for (file, _) in &files()[1..] {
            assert!(
                skill.contains(&format!("]({file})")),
                "SKILL.md does not point at {file}"
            );
        }
    }
}
