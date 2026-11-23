//! The files of a new Python Ambler project, and the `[project.scripts]` entries that run walks.
//!
//! The engine, `src/<package>/engine.py`, is ambler-py's own `src/ambler/__init__.py`, copied
//! unchanged from <https://github.com/argenkiwi/ambler-py> at commit `7bcb4dc`.

use crate::scaffold::context::{context, within};
use crate::scaffold::{Change, names};

/// The engine every Python project shares.
pub const ENGINE: &str = include_str!("../../../templates/py/engine.py");
const PYPROJECT: &str = include_str!("../../../templates/py/pyproject.toml");
const GITIGNORE: &str = include_str!("../../../templates/py/gitignore");
const README: &str = include_str!("../../../templates/py/README.md");

/// The docstrings of the package and its three subpackages, when there is nothing else in them.
const PACKAGES: &[(&str, &str)] = &[
    (
        "__init__.py",
        "\"\"\"An Ambler project: run `amble help` for how it is laid out.\"\"\"\n",
    ),
    (
        "nodes/__init__.py",
        "\"\"\"Reusable atomic step nodes and the shared state protocols they require.\"\"\"\n",
    ),
    (
        "walks/__init__.py",
        "\"\"\"Walks: state machine graphs of nodes wired together.\"\"\"\n",
    ),
    (
        "utils/__init__.py",
        "\"\"\"Side-effectful utilities (I/O, external APIs, system services).\"\"\"\n",
    ),
];

/// The counter walk, by path: under the package if it starts with `src/`.
const EXAMPLE: &[(&str, &str)] = &[
    (
        "src/nodes/__init__.py",
        include_str!("../../../templates/py/example/nodes/__init__.py"),
    ),
    (
        "src/nodes/count.py",
        include_str!("../../../templates/py/example/nodes/count.py"),
    ),
    (
        "src/nodes/start.py",
        include_str!("../../../templates/py/example/nodes/start.py"),
    ),
    (
        "src/nodes/stop.py",
        include_str!("../../../templates/py/example/nodes/stop.py"),
    ),
    (
        "src/utils/terminal.py",
        include_str!("../../../templates/py/example/utils/terminal.py"),
    ),
    (
        "src/walks/counter.py",
        include_str!("../../../templates/py/example/walks/counter.py"),
    ),
    (
        "tests/nodes/test_count.py",
        include_str!("../../../templates/py/example/tests/nodes/test_count.py"),
    ),
    (
        "tests/nodes/test_start.py",
        include_str!("../../../templates/py/example/tests/nodes/test_start.py"),
    ),
    (
        "tests/nodes/test_stop.py",
        include_str!("../../../templates/py/example/tests/nodes/test_stop.py"),
    ),
    (
        "tests/walks/test_counter.py",
        include_str!("../../../templates/py/example/tests/walks/test_counter.py"),
    ),
    (
        "specs/counter.md",
        include_str!("../../../templates/counter.md"),
    ),
];

/// The example's files, by their path in a project whose package is `package`, with the package
/// named in their imports.
#[must_use]
pub fn example(package: &str) -> Vec<(String, String)> {
    EXAMPLE
        .iter()
        .map(|(path, contents)| {
            let path = path.strip_prefix("src/").map_or_else(
                || (*path).to_string(),
                |rest| format!("src/{package}/{rest}"),
            );
            (path, contents.replace("{{package}}", package))
        })
        .collect()
}

/// The script that runs walk `name` in package `package`: `<kebab> = "<package>.walks.<name>:main"`.
#[must_use]
pub fn script(package: &str, name: &str) -> String {
    format!("{} = \"{package}.walks.{name}:main\"", names::kebab(name))
}

/// `pyproject.toml` with a script that runs the walk `name` (`uv run <name>`), in alphabetical
/// order in `[project.scripts]`, which it adds after `[project]` if there is none. `None` if the
/// script is already there or there is no `[project]` table.
#[must_use]
pub fn add_script(pyproject: &str, package: &str, name: &str) -> Option<String> {
    let entry = script(package, name);
    let key = format!("{} ", names::kebab(name));
    let mut lines: Vec<String> = pyproject.lines().map(str::to_string).collect();
    let is_table = |line: &String| line.trim_start().starts_with('[');
    let table = |wanted: &str| lines.iter().position(|line| line.trim() == wanted);
    let start = match table("[project.scripts]") {
        Some(start) => start,
        None => {
            let project = table("[project]")?;
            let end = lines[project + 1..]
                .iter()
                .position(is_table)
                .map_or(lines.len(), |index| project + 1 + index);
            // Keep one blank line between the tables.
            let at = if end > 0 && lines[end - 1].trim().is_empty() {
                end - 1
            } else {
                end
            };
            lines.splice(at..at, [String::new(), "[project.scripts]".to_string()]);
            at + 1
        }
    };
    let end = lines[start + 1..]
        .iter()
        .position(|line| is_table(line) || line.trim().is_empty())
        .map_or(lines.len(), |index| start + 1 + index);
    let body = &lines[start + 1..end];
    if body.iter().any(|line| line.trim_start().starts_with(&key)) {
        return None;
    }
    let at = body
        .iter()
        .position(|line| line.trim_start() > entry.as_str())
        .map_or(end, |index| start + 1 + index);
    lines.insert(at, entry);
    let mut out = lines.join("\n");
    if pyproject.ends_with('\n') {
        out.push('\n');
    }
    Some(out)
}

/// Every file of a new project named `project` (its import name is `package`) in `folder` (the
/// current folder if empty), with the counter walk if `example`.
#[must_use]
pub fn project(project: &str, package: &str, folder: &str, example: bool) -> Vec<Change> {
    let at = |path: &str| within(folder, path);
    let named = |template: &str| {
        template
            .replace("{{project}}", project)
            .replace("{{package}}", package)
    };
    let mut pyproject = named(PYPROJECT);
    if example {
        pyproject = add_script(&pyproject, package, "counter").unwrap_or(pyproject);
    }
    let mut changes = vec![
        Change::write(at("pyproject.toml"), pyproject),
        Change::write(at(".gitignore"), GITIGNORE),
        Change::write(at("README.md"), named(README)),
        Change::write(at(&format!("src/{package}/engine.py")), ENGINE),
        Change::write(at(&format!("src/{package}/py.typed")), ""),
    ];
    changes.extend(context(folder));
    let files = if example {
        self::example(package)
    } else {
        Vec::new()
    };
    for (path, contents) in PACKAGES {
        let path = format!("src/{package}/{path}");
        if files.iter().all(|(file, _)| *file != path) {
            changes.push(Change::write(at(&path), *contents));
        }
    }
    if files.is_empty() {
        changes.push(Change::write(at("tests/.gitkeep"), ""));
    }
    for (path, contents) in files {
        changes.push(Change::write(at(&path), contents));
    }
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
    fn an_empty_project_has_the_engine_the_package_and_the_context_files() {
        let changes = project("my-app", "my_app", "my-app", false);

        let pyproject = contents(&changes, "my-app/pyproject.toml");
        assert!(pyproject.contains("name = \"my-app\"\n"));
        assert!(pyproject.contains("reportMatchNotExhaustive = true"));
        assert!(!pyproject.contains("{{"));
        assert!(contents(&changes, "my-app/src/my_app/engine.py").contains("class Walk[Id, S]"));
        assert!(contents(&changes, "my-app/src/my_app/nodes/__init__.py").starts_with("\"\"\""));
        assert!(contents(&changes, "my-app/README.md").contains("src/my_app/engine.py"));
        assert!(contents(&changes, "my-app/AGENTS.md").contains("amble check"));
        assert_eq!(contents(&changes, "my-app/tests/.gitkeep"), "");
        assert_eq!(contents(&changes, "my-app/specs/.gitkeep"), "");
    }

    #[test]
    fn the_example_adds_the_counter_walk_its_tests_and_its_script() {
        let changes = project("my-app", "my_app", "", true);

        let walk = contents(&changes, "src/my_app/walks/counter.py");
        assert!(walk.contains("from my_app.engine import Walk\n"));
        assert!(contents(&changes, "src/my_app/nodes/__init__.py").contains("class HasCount"));
        assert!(contents(&changes, "tests/nodes/test_start.py").contains("from my_app.nodes"));
        assert!(contents(&changes, "specs/counter.md").contains("### Start"));
        assert!(
            contents(&changes, "pyproject.toml")
                .contains("[project.scripts]\ncounter = \"my_app.walks.counter:main\"\n\n")
        );
        assert!(changes.iter().all(|change| !change.path().contains("{{")));
        assert!(
            changes
                .iter()
                .all(|change| change.path() != "tests/.gitkeep")
        );
    }

    #[test]
    fn adds_walk_scripts_in_order() {
        let pyproject = "[project]\nname = \"a\"\n\n[project.scripts]\n\n[tool.x]\n";

        let one = add_script(pyproject, "a", "my_walk").unwrap();
        assert_eq!(
            one,
            "[project]\nname = \"a\"\n\n[project.scripts]\nmy-walk = \"a.walks.my_walk:main\"\n\n\
             [tool.x]\n"
        );
        let two = add_script(&one, "a", "counter").unwrap();
        assert!(two.contains(
            "[project.scripts]\ncounter = \"a.walks.counter:main\"\nmy-walk = \
             \"a.walks.my_walk:main\"\n\n"
        ));
        assert_eq!(add_script(&two, "a", "counter"), None);
        assert_eq!(add_script("[tool.x]\n", "a", "counter"), None);
    }

    #[test]
    fn adds_the_scripts_table_after_the_project_table() {
        let pyproject = "[project]\nname = \"a\"\n\n[build-system]\n";

        assert_eq!(
            add_script(pyproject, "a", "go").unwrap(),
            "[project]\nname = \"a\"\n\n[project.scripts]\ngo = \"a.walks.go:main\"\n\n\
             [build-system]\n"
        );
    }
}
