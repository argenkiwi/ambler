//! The files every new project gets whatever its target: the agent context and the specs folder.
//!
//! The context files are this repository's own: the agent instructions a new project gets are
//! the ones ambler itself is developed under, so they must not depend on the target.

use super::Change;

/// The always-loaded agent protocol. Also printed by `ambler help agents`.
pub const AGENTS: &str = include_str!("../../AGENTS.md");
const CLAUDE: &str = include_str!("../../CLAUDE.md");
/// The one skill that routes Ambler work to this command line.
pub const SKILL: &str = include_str!("../../.agents/skills/ambler/SKILL.md");

/// `path` inside `folder`, or as is if `folder` is empty (the current folder).
#[must_use]
pub fn within(folder: &str, path: &str) -> String {
    if folder.is_empty() {
        path.to_string()
    } else {
        format!("{folder}/{path}")
    }
}

/// The context files, the shared skills link and the specs folder of a project in `folder`.
#[must_use]
pub fn context(folder: &str) -> Vec<Change> {
    let at = |path: &str| within(folder, path);
    vec![
        Change::write(at("AGENTS.md"), AGENTS),
        Change::write(at("CLAUDE.md"), CLAUDE),
        Change::write(at(".agents/skills/ambler/SKILL.md"), SKILL),
        Change::Symlink {
            path: at(".claude/skills"),
            target: "../.agents/skills".to_string(),
        },
        // Specs come first in the usual order of work, so their folder is there from the start.
        Change::write(at("specs/.gitkeep"), ""),
    ]
}
