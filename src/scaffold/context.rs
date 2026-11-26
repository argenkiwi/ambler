//! The files every new project gets whatever its target: the agent context and the specs folder.
//!
//! The agent instructions a new project gets live in `templates/AGENTS.md` and
//! `templates/SKILL.md`, apart from this repository's own `AGENTS.md` and global skill, so they
//! must not depend on ambler's own policies. They name the project's own CLI, not `ambler`, so
//! an agent can follow them where only the project's dependencies are installed (a container).

use super::Change;
use super::target::Target;

/// The always-loaded agent protocol, with `<cli>`, `<engine>` and `<setup>` to fill per target.
pub const AGENTS: &str = include_str!("../../templates/AGENTS.md");
const CLAUDE: &str = include_str!("../../CLAUDE.md");
/// The skill a project gets: it routes Ambler work to the project's own CLI.
pub const SKILL: &str = include_str!("../../templates/SKILL.md");
/// The skill installed globally (`npx skills add`): it starts with `ambler new`.
pub const GLOBAL_SKILL: &str = include_str!("../../.agents/skills/ambler/SKILL.md");

/// How the project's own CLI is run in a `target` project.
#[must_use]
pub fn cli(target: Target) -> &'static str {
    match target {
        Target::Rust => "cargo ambler",
        Target::TypeScript => "bun run ambler",
        Target::Python { .. } => "uv run ambler",
    }
}

/// `template` with its placeholders filled for `target`.
#[must_use]
pub fn render(template: &str, target: Target) -> String {
    let (engine, setup) = match target {
        Target::Rust => ("`src/engine.rs`", "`cargo build`"),
        Target::TypeScript => ("`ambler.ts`", "`bun install`"),
        Target::Python { .. } => ("`src/<package>/engine.py`", "`uv sync`"),
    };
    template
        .replace("<cli>", cli(target))
        .replace("`<engine>`", engine)
        .replace("`<setup>`", setup)
}

/// `path` inside `folder`, or as is if `folder` is empty (the current folder).
#[must_use]
pub fn within(folder: &str, path: &str) -> String {
    if folder.is_empty() {
        path.to_string()
    } else {
        format!("{folder}/{path}")
    }
}

/// The context files, the shared skills link and the specs folder of a `target` project in
/// `folder`.
#[must_use]
pub fn context(folder: &str, target: Target) -> Vec<Change> {
    let at = |path: &str| within(folder, path);
    vec![
        Change::write(at("AGENTS.md"), render(AGENTS, target)),
        Change::write(at("CLAUDE.md"), CLAUDE),
        Change::write(at(".agents/skills/ambler/SKILL.md"), render(SKILL, target)),
        Change::Symlink {
            path: at(".claude/skills"),
            target: "../.agents/skills".to_string(),
        },
        // Specs come first in the usual order of work, so their folder is there from the start.
        Change::write(at("specs/.gitkeep"), ""),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const TARGETS: [Target; 3] = [
        Target::Rust,
        Target::TypeScript,
        Target::Python { package: "app" },
    ];

    #[test]
    fn the_agent_instructions_do_not_assume_a_release_policy() {
        assert!(!AGENTS.contains("Bump the version"));
    }

    #[test]
    fn the_context_files_name_the_projects_own_cli_and_need_no_global_ambler() {
        for target in TARGETS {
            for template in [AGENTS, SKILL] {
                let text = render(template, target);
                assert!(!text.contains("<cli>"), "{target:?}");
                assert!(!text.contains("<setup>") && !text.contains("<engine>"));
                assert!(!text.contains("cargo install"), "{target:?}");
                assert!(!text.contains("`ambler "), "{target:?}");
                assert!(text.contains(&format!("`{} check`", cli(target))));
            }
        }
    }
}
