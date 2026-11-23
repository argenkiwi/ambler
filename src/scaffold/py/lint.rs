//! The structural conventions of a Python Ambler project, checked against its files.
//!
//! Each rule has an id, explained in `amble help check`.

use super::source::{Module, parse};
use crate::scaffold::lint::{Violation, headings, spec_nodes, spec_sections};
use crate::scaffold::names;
use crate::scaffold::target::{Target, package_of, pyproject_name};
use std::collections::BTreeMap;

/// What a walk may list in `__all__`: the rest of the module is for its wiring test.
const WALK_SURFACE: [&str; 3] = ["Error", "main", "run"];

struct Linter<'a> {
    files: &'a BTreeMap<String, String>,
    package: String,
    found: Vec<Violation>,
}

/// The stem of `path` if it is a `.py` file directly in `folder`, other than `__init__.py`.
fn member<'a>(path: &'a str, folder: &str) -> Option<&'a str> {
    path.strip_prefix(folder)?
        .strip_prefix('/')?
        .strip_suffix(".py")
        .filter(|stem| !stem.contains('/') && *stem != "__init__")
}

impl Linter<'_> {
    fn report(&mut self, path: &str, line: usize, rule: &'static str, message: impl Into<String>) {
        self.found.push(Violation::new(path, line, rule, message));
    }

    fn target(&self) -> Target {
        // A leaked name per lint is harmless: `amble check` lints once.
        Target::Python {
            package: Box::leak(self.package.clone().into_boxed_str()),
        }
    }

    /// Flags what `amble node` and `amble walk` left unfinished.
    fn unfinished(&mut self, path: &str, source: &str) {
        let markers = self.target().unfinished();
        for (index, line) in source.lines().enumerate() {
            if markers.iter().any(|marker| line.contains(marker)) {
                self.report(path, index + 1, "todo", "still to be written");
            }
        }
    }

    /// Parses a file, reporting it if it is not valid Python.
    fn parsed(&mut self, path: &str, source: &str) -> Option<Module> {
        self.unfinished(path, source);
        match parse(source) {
            Ok(module) => Some(module),
            Err((line, error)) => {
                self.report(path, line, "parse", error);
                None
            }
        }
    }

    fn named(&mut self, path: &str, stem: &str) -> bool {
        let good = names::is_snake(stem);
        if !good {
            self.report(
                path,
                1,
                "file-name",
                "name modules in snake_case: lowercase words joined by `_`, not a keyword",
            );
        }
        good
    }

    fn node(&mut self, path: &str, name: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        if !self.named(path, name) {
            return;
        }
        let class = names::pascal(name);
        match module.class(&class) {
            None => self.report(
                path,
                1,
                "node-shape",
                format!("expected a node class `{class}`, a `@dataclass(frozen=True)`"),
            ),
            Some(node) => {
                if !node
                    .decorators
                    .iter()
                    .any(|decorator| decorator == "dataclass(frozen=True)")
                {
                    self.report(
                        path,
                        node.line,
                        "node-shape",
                        format!("`{class}` should be a `@dataclass(frozen=True)`"),
                    );
                }
                match node.method("run") {
                    Some(run) if run.is_async => {
                        let terminal = run.returns.as_deref() == Some("None");
                        if !terminal && module.class("Outcome").is_none() {
                            self.report(
                                path,
                                run.line,
                                "node-shape",
                                "expected `class Outcome(Enum)`, or `run` returning `None` for \
                                 a terminal node",
                            );
                        }
                    }
                    _ => self.report(
                        path,
                        node.line,
                        "node-shape",
                        format!("`{class}` should have `async def run(self, state)`"),
                    ),
                }
            }
        }
        if let Some(outcome) = module.class("Outcome") {
            if !outcome.has_base("Enum") {
                self.report(
                    path,
                    outcome.line,
                    "node-shape",
                    "`Outcome` should be an `Enum`",
                );
            }
            if let Some(member) = outcome
                .members
                .iter()
                .find(|member| !super::node::is_outcome_name(member))
            {
                self.report(
                    path,
                    outcome.line,
                    "node-shape",
                    format!("`{member}` is not an outcome name: UPPER_SNAKE_CASE"),
                );
            }
        }
        if !module
            .class("Utils")
            .is_some_and(|utils| utils.has_base("Protocol"))
        {
            self.report(path, 1, "node-shape", "expected `class Utils(Protocol)`");
        }
        if module.class("SystemUtils").is_none() {
            self.report(
                path,
                1,
                "node-shape",
                "expected `class SystemUtils`, the real side effects",
            );
        }
        let test = self.target().node_test_path(name).unwrap_or_default();
        if !self.files.contains_key(&test) {
            self.report(
                path,
                1,
                "node-tests",
                format!("no tests: write {test} (`amble help testing`)"),
            );
        }
    }

    fn walk(&mut self, path: &str, name: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        if !self.named(path, name) {
            return;
        }
        match &module.all {
            None => self.report(
                path,
                1,
                "walk-surface",
                "expected `__all__`, listing what of `Error`, `main` and `run` it exports",
            ),
            Some(all) => {
                if let Some(extra) = all
                    .iter()
                    .find(|name| !WALK_SURFACE.contains(&name.as_str()))
                {
                    let line = module.assignment("__all__").map_or(1, |all| all.line);
                    self.report(
                        path,
                        line,
                        "walk-surface",
                        format!(
                            "`{extra}` in `__all__`: a walk exports only `Error`, `main` and `run`"
                        ),
                    );
                }
            }
        }
        for wanted in ["main", "run"] {
            if module.function(wanted).is_none() {
                self.report(
                    path,
                    1,
                    "walk-surface",
                    format!("expected `def {wanted}()`, which runs the walk"),
                );
            }
        }
        let test = self.target().walk_test_path(name).unwrap_or_default();
        if !self.files.contains_key(&test) {
            self.report(
                path,
                1,
                "walk-tests",
                format!("no wiring test: write {test} (`amble help testing`)"),
            );
        }
        let entry = format!("\"{}.walks.{name}:main\"", self.package);
        let runs = self
            .files
            .get("pyproject.toml")
            .is_some_and(|pyproject| pyproject.contains(&entry));
        if !runs {
            self.report(
                path,
                1,
                "walk-runner",
                format!(
                    "no script runs it: add `{} = {entry}` to `[project.scripts]` in pyproject.toml",
                    names::kebab(name)
                ),
            );
        }
        let spec = format!("specs/{name}.md");
        if self.files.contains_key(&spec) {
            let ids: Vec<String> = module
                .class("Id")
                .map(|id| id.members.clone())
                .unwrap_or_default();
            // Headings lose their spaces before they are compared: `Read Move` → `ReadMove`. A
            // member matches the heading that spells it in any case, so `Check M3U File` is
            // `CHECK_M3U_FILE`.
            let sections: Vec<String> = headings(&self.files[&spec])
                .into_iter()
                .filter_map(|(_, heading)| Some(heading.strip_prefix("### ")?.replace(' ', "")))
                .collect();
            let ids: Vec<String> = ids
                .iter()
                .map(|id| {
                    let bare = id.replace('_', "");
                    sections
                        .iter()
                        .find(|section| section.eq_ignore_ascii_case(&bare))
                        .cloned()
                        .unwrap_or_else(|| names::pascal(&id.to_lowercase()))
                })
                .collect();
            self.found
                .extend(spec_nodes(self.files, &spec, path, &ids, |id| {
                    format!("Id.{}", names::shout(&names::snake(id)))
                }));
        } else {
            self.report(
                path,
                1,
                "walk-spec",
                format!("no spec: write {spec} (`amble help spec`)"),
            );
        }
    }

    fn util(&mut self, path: &str, name: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        self.named(path, name);
        let package = &self.package;
        let banned: Vec<String> = ["engine", "nodes", "walks"]
            .iter()
            .map(|part| format!("{package}.{part}"))
            .collect();
        let found: Vec<(usize, String)> = module
            .imports
            .iter()
            .filter(|import| {
                let full = match &import.name {
                    Some(name) if import.module == *package => format!("{package}.{name}"),
                    _ => import.module.clone(),
                };
                banned
                    .iter()
                    .any(|banned| full == *banned || full.starts_with(&format!("{banned}.")))
                    || import.module.starts_with("..")
            })
            .map(|import| (import.line, import.module.clone()))
            .collect();
        for (line, from) in found {
            self.report(
                path,
                line,
                "util-imports",
                format!("a utility imports nothing from the engine, nodes or walks (`{from}`)"),
            );
        }
    }

    /// A test is for a node or walk that exists.
    fn test(&mut self, path: &str, source: &str, group: &str, stem: &str) {
        self.unfinished(path, source);
        let Some(name) = stem.strip_prefix("test_") else {
            return;
        };
        let target = format!("src/{}/{group}/{name}.py", self.package);
        if !self.files.contains_key(&target) {
            self.report(
                path,
                1,
                "orphan-test",
                format!("{target} does not exist: rename or delete this test"),
            );
        }
    }

    fn lint(&mut self) {
        if !self.files.contains_key("pyproject.toml") {
            self.report(
                "pyproject.toml",
                1,
                "layout",
                "missing: run amble from the root of the project",
            );
            return;
        }
        let package = self.package.clone();
        let engine = format!("src/{package}/engine.py");
        if package.is_empty() || !self.files.contains_key(&engine) {
            self.report(
                &engine,
                1,
                "layout",
                "missing: the engine is written by `amble new`; restore it from a new project",
            );
        }
        let root = format!("src/{package}");
        for (path, source) in self.files {
            if !path.ends_with(".py") || *path == engine {
                continue;
            }
            if let Some(stem) = member(path, &format!("{root}/nodes")) {
                self.node(path, stem, source);
            } else if let Some(stem) = member(path, &format!("{root}/walks")) {
                self.walk(path, stem, source);
            } else if let Some(stem) = member(path, &format!("{root}/utils")) {
                self.util(path, stem, source);
            } else if let Some(stem) = member(path, "tests/nodes") {
                self.test(path, source, "nodes", stem);
            } else if let Some(stem) = member(path, "tests/walks") {
                self.test(path, source, "walks", stem);
            } else {
                self.parsed(path, source);
            }
        }
        for path in self.files.keys() {
            if path.starts_with("specs/") && path.ends_with(".md") {
                self.found.extend(spec_sections(self.files, path));
            }
        }
    }
}

/// Every convention of a Python project that `files` (by path) breaks, sorted.
#[must_use]
pub fn lint(files: &BTreeMap<String, String>) -> Vec<Violation> {
    let package = files
        .get("pyproject.toml")
        .and_then(|pyproject| pyproject_name(pyproject))
        .map(|name| package_of(&name))
        .unwrap_or_default();
    let mut linter = Linter {
        files,
        package,
        found: Vec::new(),
    };
    linter.lint();
    linter.found.sort();
    linter.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::Change;
    use crate::scaffold::py::project::project;

    fn example() -> BTreeMap<String, String> {
        project("my-app", "my_app", "", true)
            .into_iter()
            .filter_map(|change| match change {
                Change::Write { path, contents, .. } => Some((path, contents)),
                Change::Symlink { .. } => None,
            })
            .filter(|(path, _)| {
                path == "pyproject.toml"
                    || path.ends_with(".py")
                    || (path.starts_with("specs/") && path.ends_with(".md"))
            })
            .collect()
    }

    fn rules(files: &BTreeMap<String, String>) -> Vec<(String, &'static str)> {
        lint(files)
            .into_iter()
            .map(|violation| (violation.path, violation.rule))
            .collect()
    }

    #[test]
    fn the_example_project_follows_every_convention() {
        assert_eq!(lint(&example()), vec![]);
    }

    #[test]
    fn a_project_needs_its_manifest_and_engine() {
        let mut files = example();
        files.remove("src/my_app/engine.py");
        assert_eq!(
            rules(&files),
            [("src/my_app/engine.py".to_string(), "layout")]
        );

        assert_eq!(
            rules(&BTreeMap::new()),
            [("pyproject.toml".to_string(), "layout")]
        );
    }

    #[test]
    fn flags_unfinished_code_and_a_node_without_its_shape_or_tests() {
        let mut files = example();
        files.insert(
            "src/my_app/nodes/ask.py".to_string(),
            "class Ask:\n    def run(self, state: object) -> None:\n        raise \
             NotImplementedError(\"not yet implemented\")\n"
                .to_string(),
        );
        let found = rules(&files);
        let at = |rule| {
            found
                .iter()
                .filter(|(path, r)| path.ends_with("ask.py") && *r == rule)
                .count()
        };
        assert_eq!(at("todo"), 1);
        assert_eq!(at("node-tests"), 1);
        // Not frozen, `run` not async, no `Utils` and no `SystemUtils`.
        assert_eq!(at("node-shape"), 4);
    }

    #[test]
    fn checks_a_walks_surface_tests_runner_and_spec() {
        let mut files = example();
        let walk = files["src/my_app/walks/counter.py"].replace(
            "__all__ = [\"Error\", \"main\", \"run\"]",
            "__all__ = [\"Counter\", \"run\"]",
        );
        files.insert("src/my_app/walks/counter.py".to_string(), walk);
        files.insert(
            "pyproject.toml".to_string(),
            files["pyproject.toml"].replace("counter = \"my_app.walks.counter:main\"\n", ""),
        );
        files.remove("tests/walks/test_counter.py");
        let spec = files["specs/counter.md"].replace("### Stop", "### Halt");
        files.insert("specs/counter.md".to_string(), spec);

        let found = rules(&files);

        let walk = "src/my_app/walks/counter.py".to_string();
        assert!(found.contains(&(walk.clone(), "walk-surface")));
        assert!(found.contains(&(walk.clone(), "walk-tests")));
        assert!(found.contains(&(walk, "walk-runner")));
        assert_eq!(
            found
                .iter()
                .filter(|(path, rule)| path == "specs/counter.md" && *rule == "spec-nodes")
                .count(),
            2
        );
    }

    #[test]
    fn flags_orphan_tests_and_utilities_that_reach_into_the_program() {
        let mut files = example();
        files.insert("tests/nodes/test_gone.py".to_string(), String::new());
        files.insert(
            "src/my_app/utils/clock.py".to_string(),
            "from my_app.nodes import HasCount\nfrom my_app import engine\n".to_string(),
        );
        files.insert("src/my_app/utils/BadName.py".to_string(), String::new());

        let found = rules(&files);

        assert!(found.contains(&("tests/nodes/test_gone.py".to_string(), "orphan-test")));
        assert_eq!(
            found
                .iter()
                .filter(|(_, rule)| *rule == "util-imports")
                .count(),
            2
        );
        assert!(found.contains(&("src/my_app/utils/BadName.py".to_string(), "file-name")));
    }
}
