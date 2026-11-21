//! The structural conventions of a TypeScript Ambler project, checked against its files.
//!
//! Each rule has an id, explained in `amble help check`.

use super::source::{Kind, Module, parse};
use crate::scaffold::lint::{Violation, spec_nodes, spec_sections};
use crate::scaffold::names;
use crate::scaffold::target::Target;
use std::collections::BTreeMap;

/// What a walk may export: the rest of the file is its own business.
const WALK_SURFACE: [&str; 4] = ["State", "NodeId", "WalkUtils", "createWalk"];

struct Linter<'a> {
    files: &'a BTreeMap<String, String>,
    found: Vec<Violation>,
}

/// The stem of `path` if it is a `.ts` file directly in `<group>/`.
fn member<'a>(path: &'a str, group: &str) -> Option<&'a str> {
    path.strip_prefix(group)?
        .strip_prefix('/')?
        .strip_suffix(".ts")
        .filter(|stem| !stem.contains('/'))
}

/// The stem of `path` if it is a test directly in `<group>/tests/`.
fn test_of<'a>(path: &'a str, group: &str) -> Option<&'a str> {
    path.strip_prefix(group)?
        .strip_prefix("/tests/")?
        .strip_suffix(".test.ts")
        .filter(|stem| !stem.contains('/'))
}

/// The `snake_case` name of a file stem, if the stem is `kebab-case`.
fn name_of(stem: &str) -> Option<String> {
    let snake = stem.replace('-', "_");
    (names::is_snake(&snake) && names::kebab(&snake) == stem).then_some(snake)
}

impl Linter<'_> {
    fn report(&mut self, path: &str, line: usize, rule: &'static str, message: impl Into<String>) {
        self.found.push(Violation::new(path, line, rule, message));
    }

    /// Flags what `amble node` and `amble walk` left unfinished.
    fn unfinished(&mut self, path: &str, source: &str) {
        let markers = Target::TypeScript.unfinished();
        for (index, line) in source.lines().enumerate() {
            if markers.iter().any(|marker| line.contains(marker)) {
                self.report(path, index + 1, "todo", "still to be written");
            }
        }
    }

    /// Parses a file, reporting it if it is not valid TypeScript.
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

    fn named(&mut self, path: &str, stem: &str) -> Option<String> {
        let name = name_of(stem);
        if name.is_none() {
            self.report(
                path,
                1,
                "file-name",
                "name files in kebab-case: lowercase words joined by `-`",
            );
        }
        name
    }

    fn node(&mut self, path: &str, stem: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        let Some(name) = self.named(path, stem) else {
            return;
        };
        if !module
            .exported("factory")
            .is_some_and(|item| item.kind == Kind::Const)
        {
            self.report(
                path,
                1,
                "node-shape",
                "expected `export const factory: NodeFactory<State, Edge, Utils>`",
            );
        }
        if module.exported("Edge").is_none() || module.union("Edge").is_none() {
            self.report(
                path,
                1,
                "node-shape",
                "expected `export type Edge`, a union of `on…` string literals",
            );
        } else if let Some(edge) = module
            .union("Edge")
            .unwrap_or_default()
            .iter()
            .find(|edge| !super::node::is_edge_name(edge))
        {
            let line = module.item("Edge").map_or(1, |item| item.line);
            self.report(
                path,
                line,
                "node-shape",
                format!("`{edge}` is not an edge name: `on` and then PascalCase"),
            );
        }
        for wanted in ["State", "Utils"] {
            if module.exported(wanted).is_none() {
                self.report(
                    path,
                    1,
                    "node-shape",
                    format!("expected `export` of `{wanted}`"),
                );
            }
        }
        let test = Target::TypeScript.node_test_path(&name).unwrap_or_default();
        if !self.files.contains_key(&test) {
            self.report(
                path,
                1,
                "node-tests",
                format!("expected its tests in {test}, one per edge"),
            );
        }
    }

    fn walk(&mut self, path: &str, stem: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        let Some(name) = self.named(path, stem) else {
            return;
        };
        for item in module.items.iter().filter(|item| item.exported) {
            if !WALK_SURFACE.contains(&item.name.as_str()) {
                self.report(
                    path,
                    item.line,
                    "walk-surface",
                    format!(
                        "`{}` should not be exported: a walk exports only {}",
                        item.name,
                        WALK_SURFACE.map(|name| format!("`{name}`")).join(", ")
                    ),
                );
            }
        }
        if module.exported("createWalk").is_none() {
            self.report(
                path,
                1,
                "walk-surface",
                "expected `export function createWalk`",
            );
        }
        let test = Target::TypeScript.walk_test_path(&name).unwrap_or_default();
        if !self.files.contains_key(&test) {
            self.report(
                path,
                1,
                "walk-tests",
                format!("expected a test of the wiring in {test}"),
            );
        }
        let script = format!("bun {path}");
        let has_runner = self
            .files
            .get("package.json")
            .is_some_and(|package| package.contains(&script));
        if !has_runner {
            self.report(
                path,
                1,
                "walk-runner",
                format!("no script in package.json runs `{script}`"),
            );
        }
        let spec = format!("specs/{name}.md");
        if !self.files.contains_key(&spec) {
            self.report(path, 1, "walk-spec", format!("expected its spec in {spec}"));
        } else if let Some(ids) = module.union("NodeId") {
            // A spec's `### Check Config` is the node `CHECK_CONFIG`.
            let ids: Vec<String> = ids
                .iter()
                .map(|id| names::pascal(&id.to_ascii_lowercase()))
                .collect();
            let show: fn(&str) -> String = |id| format!("\"{}\"", names::shout(&names::snake(id)));
            self.found
                .extend(spec_nodes(self.files, &spec, path, &ids, show));
        }
    }

    fn util(&mut self, path: &str, source: &str) {
        let Some(module) = self.parsed(path, source) else {
            return;
        };
        self.named(
            path,
            path.trim_start_matches("utils/").trim_end_matches(".ts"),
        );
        for import in &module.imports {
            let reaches_up = import.source == "../ambler.ts"
                || import.source.starts_with("../nodes/")
                || import.source.starts_with("../walks/");
            if reaches_up {
                self.report(
                    path,
                    import.line,
                    "util-imports",
                    format!(
                        "`{}`: utilities know nothing of nodes, walks or the engine",
                        import.source
                    ),
                );
            }
        }
    }

    /// A test file whose node or walk is gone.
    fn orphan(&mut self, path: &str, group: &str, stem: &str) {
        let subject = format!("{group}/{stem}.ts");
        if !self.files.contains_key(&subject) {
            self.report(path, 1, "orphan-test", format!("{subject} does not exist"));
        }
    }
}

/// Checks the conventions over `files`: paths relative to the project root, with contents.
///
/// Expects `package.json`, `ambler.ts`, every `.ts` file under `nodes/`, `walks/` and `utils/`,
/// and every `.md` file under `specs/`.
#[must_use]
pub fn lint(files: &BTreeMap<String, String>) -> Vec<Violation> {
    let mut linter = Linter {
        files,
        found: Vec::new(),
    };
    let missing: Vec<&str> = ["package.json", "ambler.ts"]
        .into_iter()
        .filter(|path| !files.contains_key(*path))
        .collect();
    if !missing.is_empty() {
        for path in missing {
            linter.report(
                path,
                1,
                "layout",
                "missing: run `amble check` from the root of an Ambler project",
            );
        }
        linter.found.sort();
        return linter.found;
    }
    for (path, source) in files {
        if let Some(stem) = member(path, "nodes") {
            linter.node(path, stem, source);
        } else if let Some(stem) = member(path, "walks") {
            linter.walk(path, stem, source);
        } else if member(path, "utils").is_some() {
            linter.util(path, source);
        } else if let Some(stem) = test_of(path, "nodes") {
            linter.unfinished(path, source);
            linter.orphan(path, "nodes", stem);
        } else if let Some(stem) = test_of(path, "walks") {
            linter.unfinished(path, source);
            linter.orphan(path, "walks", stem);
        } else if path.starts_with("specs/") && path.ends_with(".md") {
            linter.found.extend(spec_sections(files, path));
        }
    }
    linter.found.sort();
    linter.found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::Change;
    use crate::scaffold::ts::project::project;

    /// The files of the example project, as `amble check` reads them.
    fn example() -> BTreeMap<String, String> {
        project("demo", "", true)
            .into_iter()
            .filter_map(|change| match change {
                Change::Write { path, contents, .. }
                    if path.ends_with(".ts")
                        || path.ends_with(".md") && path.starts_with("specs/")
                        || path == "package.json" =>
                {
                    Some((path, contents))
                }
                _ => None,
            })
            .collect()
    }

    fn rules(files: &BTreeMap<String, String>) -> Vec<String> {
        lint(files).iter().map(ToString::to_string).collect()
    }

    #[test]
    fn the_example_project_follows_every_convention() {
        assert_eq!(rules(&example()), Vec::<String>::new());
    }

    #[test]
    fn outside_a_project_says_so_once_per_marker() {
        assert_eq!(
            rules(&BTreeMap::new()),
            [
                "ambler.ts:1: layout: missing: run `amble check` from the root of an Ambler project",
                "package.json:1: layout: missing: run `amble check` from the root of an Ambler project",
            ]
        );
    }

    #[test]
    fn a_fresh_node_is_unfinished_until_written() {
        let mut files = example();
        let spec = crate::scaffold::ts::node::NodeSpec {
            name: "greet".to_string(),
            edges: vec!["onDone".to_string()],
            terminal: true,
            state: vec![],
        };
        files.insert(
            "nodes/greet.ts".to_string(),
            crate::scaffold::ts::node::node(&spec),
        );
        files.insert(
            "nodes/tests/greet.test.ts".to_string(),
            crate::scaffold::ts::node::tests(&spec),
        );

        let found = rules(&files);

        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("nodes/greet.ts:"));
        assert!(found[0].contains(": todo: still to be written"));
    }

    #[test]
    fn nodes_need_their_contract_their_tests_and_a_kebab_name() {
        let mut files = example();
        files.insert(
            "nodes/askName.ts".to_string(),
            "export const factory = 1;\n".to_string(),
        );
        files.insert(
            "nodes/ask.ts".to_string(),
            "export const factory = 1;\nexport type Edge = \"done\";\n".to_string(),
        );
        files.insert("nodes/tests/gone.test.ts".to_string(), String::new());

        let found = rules(&files);

        assert!(found.contains(
            &"nodes/askName.ts:1: file-name: name files in kebab-case: lowercase words joined by `-`"
                .to_string()
        ));
        assert!(
            found.contains(
                &"nodes/ask.ts:2: node-shape: `done` is not an edge name: `on` and then PascalCase"
                    .to_string()
            )
        );
        assert!(
            found.contains(&"nodes/ask.ts:1: node-shape: expected `export` of `State`".to_string())
        );
        assert!(found.contains(
            &"nodes/ask.ts:1: node-tests: expected its tests in nodes/tests/ask.test.ts, one per edge"
                .to_string()
        ));
        assert!(found.contains(
            &"nodes/tests/gone.test.ts:1: orphan-test: nodes/gone.ts does not exist".to_string()
        ));
    }

    #[test]
    fn walks_export_little_run_from_a_script_and_match_their_spec() {
        let mut files = example();
        let walk = files["walks/counter.ts"].replace(
            "export function createWalk",
            "export const extra = 1;\n\nexport function createWalk",
        );
        files.insert("walks/counter.ts".to_string(), walk);
        files.insert(
            "package.json".to_string(),
            files["package.json"].replace("bun walks/counter.ts", "bun other.ts"),
        );
        files.insert(
            "specs/counter.md".to_string(),
            files["specs/counter.md"].replace("### Stop", "### Halt"),
        );

        let found = rules(&files);

        assert!(
            found
                .iter()
                .any(|v| v.contains("walk-surface: `extra` should not be exported"))
        );
        assert!(found.contains(
            &"walks/counter.ts:1: walk-runner: no script in package.json runs `bun walks/counter.ts`"
                .to_string()
        ));
        assert!(found.iter().any(|v| v.starts_with("specs/counter.md:")
            && v.contains(
                "spec-nodes: this node is not wired in walks/counter.ts (no `\"HALT\"`)"
            )));
        assert!(found.contains(
            &"specs/counter.md:1: spec-nodes: `\"STOP\"` in walks/counter.ts has no `###` section here"
                .to_string()
        ));
    }

    #[test]
    fn utilities_do_not_reach_into_nodes_walks_or_the_engine() {
        let mut files = example();
        files.insert(
            "utils/clock.ts".to_string(),
            "import type { Node } from \"../ambler.ts\";\n\nexport type T = Node<number, \"A\">;\n"
                .to_string(),
        );

        assert_eq!(
            rules(&files),
            [
                "utils/clock.ts:1: util-imports: `../ambler.ts`: utilities know nothing of nodes, \
              walks or the engine"
            ]
        );
    }
}
