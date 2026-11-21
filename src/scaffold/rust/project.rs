//! The files of a new Rust Ambler project.
//!
//! The engine is this repository's own: what `amble new` writes is what amble itself compiles
//! and tests.

use super::register::{append, register};
use super::{node, walk};
use crate::scaffold::Change;
use crate::scaffold::context::{context, within};

const ENGINE: &str = include_str!("../../engine.rs");

const CARGO: &str = include_str!("../../../templates/rust/Cargo.toml");
const GITIGNORE: &str = include_str!("../../../templates/rust/gitignore");
const README: &str = include_str!("../../../templates/rust/README.md");
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

/// The name code uses for the crate a `Cargo.toml` declares: the package name with `-` as `_`.
#[must_use]
pub fn crate_name(cargo_toml: &str) -> Option<String> {
    cargo_toml
        .lines()
        .skip_while(|line| line.trim() != "[package]")
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "name").then(|| value.trim().trim_matches('"').replace('-', "_"))
        })
}

fn registered(source: &str, name: &str) -> String {
    register(source, name).unwrap_or_else(|| source.to_string())
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
    ];
    changes.extend(context(folder));

    let (mut nodes, mut walks, mut utils) =
        (NODES.to_string(), WALKS.to_string(), UTILS.to_string());
    if example {
        for (name, source) in EXAMPLE_NODES {
            nodes = registered(&nodes, name);
            changes.push(Change::write(at(&format!("src/nodes/{name}.rs")), *source));
        }
        nodes = append(&nodes, &node::has_trait("count", "i32"));
        utils = registered(&utils, "terminal");
        walks = registered(&walks, "counter");
        let crate_name = package.replace('-', "_");
        changes.extend([
            Change::write(at("src/utils/terminal.rs"), EXAMPLE_UTIL),
            Change::write(at("src/walks/counter.rs"), EXAMPLE_WALK),
            Change::write(at("specs/counter.md"), EXAMPLE_SPEC),
            Change::write(at("src/main.rs"), walk::runner(&crate_name, "counter")),
        ]);
    }
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
    fn reads_the_crate_name_from_the_package_table() {
        let toml = "[workspace]\nname = \"no\"\n\n[package]\nname = \"my-app\"\n\n[dependencies]\n";
        assert_eq!(crate_name(toml).as_deref(), Some("my_app"));
        assert_eq!(crate_name("[dependencies]\nname = \"no\"\n"), None);
    }

    #[test]
    fn an_empty_project_has_the_engine_the_folders_and_the_context_files() {
        let changes = project("my-app", "my-app", false);

        assert!(contents(&changes, "my-app/Cargo.toml").contains("name = \"my-app\"\n"));
        assert!(!contents(&changes, "my-app/Cargo.toml").contains("{{"));
        assert!(contents(&changes, "my-app/src/engine.rs").contains("pub trait Walk"));
        assert!(contents(&changes, "my-app/src/lib.rs").contains("pub use engine::"));
        assert_eq!(contents(&changes, "my-app/src/nodes/mod.rs"), NODES);
        assert_eq!(contents(&changes, "my-app/specs/.gitkeep"), "");
        assert!(contents(&changes, "my-app/AGENTS.md").contains("amble check"));
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
