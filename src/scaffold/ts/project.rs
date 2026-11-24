//! The files of a new TypeScript Ambler project, and the `package.json` scripts that run walks.
//!
//! The engine, `ambler.ts`, is ambler-ts's own, copied unchanged from
//! <https://github.com/argenkiwi/ambler-ts> at commit `354c8a2`.

use crate::scaffold::context::{context, within};
use crate::scaffold::{Change, names};

/// The engine every TypeScript project shares.
pub const ENGINE: &str = include_str!("../../../templates/ts/ambler.ts");
const CLI: &str = include_str!("../../../templates/ts/cli.ts");
const PACKAGE: &str = include_str!("../../../templates/ts/package.json");
const TSCONFIG: &str = include_str!("../../../templates/ts/tsconfig.json");
const BIOME: &str = include_str!("../../../templates/ts/biome.json");
const GITIGNORE: &str = include_str!("../../../templates/ts/gitignore");
const README: &str = include_str!("../../../templates/ts/README.md");

const EXAMPLE: &[(&str, &str)] = &[
    (
        "nodes/count.ts",
        include_str!("../../../templates/ts/example/nodes/count.ts"),
    ),
    (
        "nodes/start.ts",
        include_str!("../../../templates/ts/example/nodes/start.ts"),
    ),
    (
        "nodes/stop.ts",
        include_str!("../../../templates/ts/example/nodes/stop.ts"),
    ),
    (
        "nodes/tests/count.test.ts",
        include_str!("../../../templates/ts/example/nodes/tests/count.test.ts"),
    ),
    (
        "nodes/tests/start.test.ts",
        include_str!("../../../templates/ts/example/nodes/tests/start.test.ts"),
    ),
    (
        "nodes/tests/stop.test.ts",
        include_str!("../../../templates/ts/example/nodes/tests/stop.test.ts"),
    ),
    (
        "walks/counter.ts",
        include_str!("../../../templates/ts/example/walks/counter.ts"),
    ),
    (
        "walks/tests/counter.test.ts",
        include_str!("../../../templates/ts/example/walks/tests/counter.test.ts"),
    ),
    (
        "specs/counter.md",
        include_str!("../../../templates/counter.md"),
    ),
];

/// `package.json` with a script that runs the walk `name` (`bun run <name>`), in alphabetical
/// order among the walk scripts that follow the built-in `test`, `check` and `lint`. `None` if
/// the script is already there or the file has no `"scripts"` object.
#[must_use]
pub fn add_script(package_json: &str, name: &str) -> Option<String> {
    let kebab = names::kebab(name);
    let key = format!("\"{kebab}\":");
    let mut lines: Vec<String> = package_json.lines().map(str::to_string).collect();
    let start = lines
        .iter()
        .position(|line| line.trim_start().starts_with("\"scripts\":"))?;
    let end = start
        + lines[start..]
            .iter()
            .position(|line| line.trim_start().starts_with('}'))?;
    let body = &lines[start + 1..end];
    if body.iter().any(|line| line.trim_start().starts_with(&key)) {
        return None;
    }
    let indent: String = body.first().map_or_else(
        || "    ".to_string(),
        |line| line.chars().take_while(|c| c.is_whitespace()).collect(),
    );
    let builtin = ["\"test\":", "\"check\":", "\"lint\":"];
    let at = body
        .iter()
        .position(|line| {
            let line = line.trim_start();
            !builtin.iter().any(|known| line.starts_with(known)) && line > key.as_str()
        })
        .map_or(end, |index| start + 1 + index);
    // The line before the new one needs a comma, and the new one needs one unless it is last.
    let comma = if at == end { "" } else { "," };
    if at == end && end > start + 1 && !lines[end - 1].trim_end().ends_with(',') {
        lines[end - 1].push(',');
    }
    lines.insert(
        at,
        format!("{indent}\"{kebab}\": \"bun walks/{kebab}.ts\"{comma}"),
    );
    let mut out = lines.join("\n");
    if package_json.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Every file of a new project named `package` in `folder` (the current folder if empty), with
/// the counter walk if `example`.
#[must_use]
pub fn project(package: &str, folder: &str, example: bool) -> Vec<Change> {
    let at = |path: &str| within(folder, path);
    let named = |template: &str| template.replace("{{package}}", package);
    let mut package_json = named(PACKAGE);
    if example {
        package_json = add_script(&package_json, "counter").unwrap_or(package_json);
    }
    let mut changes = vec![
        Change::write(at("package.json"), package_json),
        Change::write(at("tsconfig.json"), TSCONFIG),
        Change::write(at("biome.json"), BIOME),
        Change::write(at(".gitignore"), GITIGNORE),
        Change::write(at("README.md"), named(README)),
        Change::write(at("ambler.ts"), ENGINE),
        Change::write(at("cli.ts"), CLI),
    ];
    changes.extend(context(folder));
    if example {
        for (path, contents) in EXAMPLE {
            changes.push(Change::write(at(path), *contents));
        }
    } else {
        changes.extend([
            Change::write(at("nodes/.gitkeep"), ""),
            Change::write(at("walks/.gitkeep"), ""),
        ]);
    }
    changes.push(Change::write(at("utils/.gitkeep"), ""));
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
    fn an_empty_project_has_the_engine_the_tools_and_the_context_files() {
        let changes = project("my-app", "my-app", false);

        let package = contents(&changes, "my-app/package.json");
        assert!(package.contains("\"name\": \"my-app\","));
        assert!(!package.contains("{{"));
        assert!(contents(&changes, "my-app/ambler.ts").contains("export function ambler"));
        assert!(contents(&changes, "my-app/tsconfig.json").contains("\"strict\": true"));
        assert!(contents(&changes, "my-app/biome.json").contains("\"preset\": \"recommended\""));
        assert!(contents(&changes, "my-app/AGENTS.md").contains("ambler check"));
        assert_eq!(contents(&changes, "my-app/nodes/.gitkeep"), "");
        assert_eq!(contents(&changes, "my-app/specs/.gitkeep"), "");
        assert!(changes.contains(&Change::Symlink {
            path: "my-app/.claude/skills".to_string(),
            target: "../.agents/skills".to_string(),
        }));
        assert!(changes.iter().all(|change| !change.path().contains("src/")));
    }

    #[test]
    fn the_example_adds_the_counter_walk_its_tests_and_its_script() {
        let changes = project("my-app", "", true);

        assert!(contents(&changes, "walks/counter.ts").contains("export function createWalk"));
        assert!(contents(&changes, "nodes/tests/count.test.ts").contains("bun:test"));
        assert!(contents(&changes, "specs/counter.md").contains("### Start"));
        assert!(
            contents(&changes, "package.json").contains(
                "\"lint\": \"biome check\",\n    \"counter\": \"bun walks/counter.ts\"\n"
            )
        );
        assert!(
            changes
                .iter()
                .all(|change| change.path() != "nodes/.gitkeep")
        );
    }

    #[test]
    fn adds_walk_scripts_in_order_after_the_built_in_ones() {
        let package = "{\n  \"scripts\": {\n    \"test\": \"bun test\",\n    \"lint\": \"biome check\"\n  }\n}\n";

        let one = add_script(package, "my_walk").unwrap();
        assert_eq!(
            one,
            "{\n  \"scripts\": {\n    \"test\": \"bun test\",\n    \"lint\": \"biome check\",\n    \
             \"my-walk\": \"bun walks/my-walk.ts\"\n  }\n}\n"
        );
        let two = add_script(&one, "counter").unwrap();
        assert!(two.contains(
            "\"lint\": \"biome check\",\n    \"counter\": \"bun walks/counter.ts\",\n    \
             \"my-walk\": \"bun walks/my-walk.ts\"\n"
        ));
        let three = add_script(&two, "zoo").unwrap();
        assert!(three.contains(
            "\"my-walk\": \"bun walks/my-walk.ts\",\n    \"zoo\": \"bun walks/zoo.ts\"\n  }"
        ));
        assert_eq!(add_script(&three, "counter"), None);
        assert_eq!(add_script("{}\n", "counter"), None);
    }
}
