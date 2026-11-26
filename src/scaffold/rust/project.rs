//! The files of a new Rust Ambler project.
//!
//! The engine is this repository's own: what `ambler new` writes is what ambler itself compiles
//! and tests.

use crate::scaffold::Change;
use crate::scaffold::context::{context, within};
use crate::scaffold::target::Target;

const ENGINE: &str = include_str!("../../engine.rs");

const CARGO: &str = include_str!("../../../templates/rust/Cargo.toml");
const GITIGNORE: &str = include_str!("../../../templates/rust/gitignore");
const README: &str = include_str!("../../../templates/rust/README.md");
const XTASK_CARGO: &str = include_str!("../../../templates/rust/xtask/Cargo.toml");
const XTASK_MAIN: &str = include_str!("../../../templates/rust/xtask/main.rs");
const CARGO_CONFIG: &str = include_str!("../../../templates/rust/cargo_config.toml");
const LIB: &str = include_str!("../../../templates/rust/lib.rs");
const NODES: &str = include_str!("../../../templates/rust/nodes_mod.rs");
const WALKS: &str = include_str!("../../../templates/rust/walks_mod.rs");
const UTILS: &str = include_str!("../../../templates/rust/utils_mod.rs");

const EXAMPLE_NODES: &[(&str, &str)] = &[
    (
        "count",
        include_str!("../../../templates/rust/example/count.rs"),
    ),
    (
        "start",
        include_str!("../../../templates/rust/example/start.rs"),
    ),
    (
        "stop",
        include_str!("../../../templates/rust/example/stop.rs"),
    ),
];
const EXAMPLE_NODES_MOD: &str = include_str!("../../../templates/rust/example/nodes_mod.rs");
const EXAMPLE_WALKS_MOD: &str = include_str!("../../../templates/rust/example/walks_mod.rs");
const EXAMPLE_UTILS_MOD: &str = include_str!("../../../templates/rust/example/utils_mod.rs");
const EXAMPLE_MAIN: &str = include_str!("../../../templates/rust/example/main.rs");
const EXAMPLE_UTIL: &str = include_str!("../../../templates/rust/example/terminal.rs");
const EXAMPLE_WALK: &str = include_str!("../../../templates/rust/example/counter.rs");
const EXAMPLE_SPEC: &str = include_str!("../../../templates/counter.md");

/// Whether `name` can be a Cargo package name and, with `-` as `_`, a crate name.
#[must_use]
pub fn is_package_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Every file of a new project named `package` in `folder` (the current folder if empty), with
/// the counter walk if `example`.
#[must_use]
pub fn project(package: &str, folder: &str, example: bool) -> Vec<Change> {
    let at = |path: &str| within(folder, path);
    let named = |template: &str| template.replace("{{package}}", package);
    let extra = if example { "rand = \"0.9\"\n" } else { "" };

    let mut changes = vec![
        Change::write(at("Cargo.toml"), named(CARGO).replace("{{extra}}", extra)),
        Change::write(at(".gitignore"), GITIGNORE),
        Change::write(at("README.md"), named(README)),
        Change::write(at("src/lib.rs"), named(LIB)),
        Change::write(at("src/engine.rs"), ENGINE),
        Change::write(at("xtask/Cargo.toml"), XTASK_CARGO),
        Change::write(at("xtask/src/main.rs"), XTASK_MAIN),
        Change::write(at(".cargo/config.toml"), CARGO_CONFIG),
    ];
    changes.extend(context(folder, Target::Rust));

    let (nodes, walks, utils) = if example {
        for (name, source) in EXAMPLE_NODES {
            changes.push(Change::write(at(&format!("src/nodes/{name}.rs")), *source));
        }
        changes.extend([
            Change::write(at("src/utils/terminal.rs"), EXAMPLE_UTIL),
            Change::write(at("src/walks/counter.rs"), EXAMPLE_WALK),
            Change::write(at("specs/counter.md"), EXAMPLE_SPEC),
            Change::write(
                at("src/main.rs"),
                EXAMPLE_MAIN.replace("{{crate}}", &package.replace('-', "_")),
            ),
        ]);
        (EXAMPLE_NODES_MOD, EXAMPLE_WALKS_MOD, EXAMPLE_UTILS_MOD)
    } else {
        (NODES, WALKS, UTILS)
    };
    changes.extend([
        Change::write(at("src/nodes/mod.rs"), nodes),
        Change::write(at("src/walks/mod.rs"), walks),
        Change::write(at("src/utils/mod.rs"), utils),
    ]);
    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contents<'a>(changes: &'a [Change], wanted: &str) -> &'a str {
        changes
            .iter()
            .find_map(|change| match change {
                Change::Write { path, contents, .. } if path == wanted => Some(contents.as_str()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{wanted} is not written"))
    }

    #[test]
    fn package_names_are_lowercase_and_may_have_dashes() {
        assert!(is_package_name("my-app"));
        assert!(!is_package_name("MyApp"));
        assert!(!is_package_name("-app"));
        assert!(!is_package_name("my app"));
    }

    #[test]
    fn an_empty_project_has_the_engine_the_folders_and_the_context_files() {
        let changes = project("my-app", "my-app", false);

        assert!(contents(&changes, "my-app/Cargo.toml").contains("name = \"my-app\"\n"));
        assert!(!contents(&changes, "my-app/Cargo.toml").contains("{{"));
        assert!(contents(&changes, "my-app/src/engine.rs").contains("pub trait Walk"));
        assert!(contents(&changes, "my-app/Cargo.toml").contains("members = [\".\", \"xtask\"]"));
        assert!(contents(&changes, "my-app/xtask/Cargo.toml").contains("ambler-rs"));
        assert!(contents(&changes, "my-app/xtask/src/main.rs").contains("ambler::cli::run"));
        assert!(contents(&changes, "my-app/.cargo/config.toml").contains("ambler = \"run"));
        assert!(contents(&changes, "my-app/src/lib.rs").contains("pub use engine::"));
        assert_eq!(contents(&changes, "my-app/src/nodes/mod.rs"), NODES);
        assert_eq!(contents(&changes, "my-app/specs/.gitkeep"), "");
        assert!(contents(&changes, "my-app/AGENTS.md").contains("ambler check"));
        assert_eq!(contents(&changes, "my-app/CLAUDE.md").trim(), "@AGENTS.md");
        assert!(contents(&changes, "my-app/.agents/skills/ambler/SKILL.md").starts_with("---\n"));
        assert!(changes.contains(&Change::Symlink {
            path: "my-app/.claude/skills".to_string(),
            target: "../.agents/skills".to_string(),
        }));
        assert!(
            changes
                .iter()
                .all(|change| change.path() != "my-app/src/main.rs")
        );
    }

    #[test]
    fn a_project_in_the_current_folder_has_root_relative_paths() {
        let changes = project("my-app", "", false);

        assert!(contents(&changes, "Cargo.toml").contains("name = \"my-app\"\n"));
        assert!(contents(&changes, "src/engine.rs").contains("pub trait Walk"));
        assert!(changes.contains(&Change::Symlink {
            path: ".claude/skills".to_string(),
            target: "../.agents/skills".to_string(),
        }));
    }

    #[test]
    fn the_example_adds_the_counter_walk_registered_and_runnable() {
        let changes = project("my-app", "my-app", true);

        assert!(contents(&changes, "my-app/Cargo.toml").contains("rand = \"0.9\"\n"));
        assert_eq!(
            contents(&changes, "my-app/src/nodes/mod.rs"),
            "//! Nodes: atomic steps, and the state traits they share.\n\n\
             pub mod count;\npub mod start;\npub mod stop;\n\n\
             /// A state that has a count field.\npub trait HasCount {\n    \
             fn count(&self) -> i32;\n    fn count_mut(&mut self) -> &mut i32;\n}\n"
        );
        assert!(contents(&changes, "my-app/src/walks/mod.rs").ends_with("pub mod counter;\n"));
        assert!(contents(&changes, "my-app/src/utils/mod.rs").ends_with("pub mod terminal;\n"));
        assert!(contents(&changes, "my-app/src/main.rs").contains("my_app::walks::counter::run()"));
        assert!(contents(&changes, "my-app/specs/counter.md").contains("### Start"));
    }
}
