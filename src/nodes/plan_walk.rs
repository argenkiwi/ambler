use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan, HasTarget};
use crate::scaffold::rust::project::crate_name;
use crate::scaffold::rust::register::register;
use crate::scaffold::rust::source::{node_info, state_traits};
use crate::scaffold::rust::walk::{WalkSpec, parse_edge, runner, spec_md, validate, walk};
use crate::scaffold::target::Target;
use crate::scaffold::{Change, Plan, args, names, py, ts};
use crate::utils::fs;
use std::convert::Infallible;
use std::io;

const USAGE: &str = "usage: amble walk <name> --nodes <start,next,...> \
                     [--edge <node>.<Outcome>=<node|halt>]... [--no-runner]";
const MODULES: &str = "src/walks/mod.rs";
const NODES: &str = "src/nodes/mod.rs";

/// Where [`PlanWalk`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The plan holds the walk, its registration, its spec and its runner.
    Planned,
    /// The arguments or the project do not allow it; the message says why.
    Rejected,
}

pub trait Utils {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> io::Result<String>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemUtils;

impl Utils for SystemUtils {
    fn exists(&self, path: &str) -> bool {
        fs::exists(path)
    }

    fn read(&self, path: &str) -> io::Result<String> {
        fs::read(path)
    }
}

/// Plans a new walk from the nodes it wires, read from `src/nodes/`.
#[derive(Clone, Debug, Default)]
pub struct PlanWalk<U = SystemUtils> {
    utils: U,
}

impl PlanWalk {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> PlanWalk<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> PlanWalk<U> {
    fn read(&self, path: &str) -> Result<String, String> {
        self.utils
            .read(path)
            .map_err(|_| format!("{path} not found: run this from the root of an Ambler project"))
    }

    fn plan(&self, target: Target, args: &[String]) -> Result<Plan, String> {
        match target {
            Target::Rust => self.plan_rust(args),
            Target::TypeScript => self.plan_ts(args),
            Target::Python { .. } => self.plan_py(target, args),
        }
    }

    fn plan_py(&self, target: Target, args: &[String]) -> Result<Plan, String> {
        let Target::Python { package } = target else {
            unreachable!("plan_py is only called for Python");
        };
        let parsed = args::parse(args, &["nodes", "edge"], &["no-runner"])
            .map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        let name = name.replace('-', "_");
        if !names::is_snake(&name) {
            return Err(format!("`{name}` is not a walk name: use snake_case"));
        }
        let nodes: Vec<String> = parsed
            .list("nodes")
            .into_iter()
            .map(|node| node.replace('-', "_"))
            .collect();
        if nodes.is_empty() {
            return Err(format!(
                "name the nodes to wire, starting node first\n{USAGE}"
            ));
        }
        let edges = parsed
            .list("edge")
            .into_iter()
            .map(parse_edge)
            .collect::<Result<Vec<_>, _>>()?;

        if !target.is_project(|path| self.utils.exists(path)) {
            return Err(
                "pyproject.toml or the package's engine.py not found: run this from the root of \
                 an Ambler project"
                    .to_string(),
            );
        }
        let path = target.walk_path(&name);
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        let missing: Vec<&String> = nodes
            .iter()
            .filter(|node| !self.utils.exists(&target.node_path(node)))
            .collect();
        if !missing.is_empty() {
            let commands: Vec<String> = missing
                .iter()
                .map(|node| format!("  amble node {node} (--outcomes <a,b> | --terminal)"))
                .collect();
            return Err(format!(
                "these nodes do not exist yet; create them first:\n{}",
                commands.join("\n")
            ));
        }
        let init_path = format!("{}/__init__.py", target.nodes_folder());
        let init = py::source::parse(&self.utils.read(&init_path).unwrap_or_default())
            .map_err(|(line, error)| format!("{init_path}:{line}: {error}"))?;
        let nodes = nodes
            .iter()
            .map(|node| {
                let path = target.node_path(node);
                py::source::node_info(node, &self.read(&path)?, &init)
                    .map_err(|e| format!("{path}: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let spec = py::walk::WalkSpec::new(&name, package, nodes, edges);
        py::walk::validate(&spec)?;
        let test_path = target.walk_test_path(&name).unwrap_or_default();
        let mut changes = vec![
            Change::source(target, &path, py::walk::walk(&spec)?),
            Change::source(target, &test_path, py::walk::tests(&spec)?),
        ];
        let spec_path = format!("specs/{name}.md");
        if !self.utils.exists(&spec_path) {
            changes.push(Change::write(spec_path, py::walk::spec_md(&spec)?));
        }
        let mut run_with = format!("`{package}.walks.{name}:main`");
        if !parsed.has("no-runner") {
            let pyproject = self.read("pyproject.toml")?;
            let script = names::kebab(&name);
            let pyproject =
                py::project::add_script(&pyproject, package, &name).ok_or_else(|| {
                    format!(
                        "pyproject.toml already has a `{script}` script, or no `[project]`; pass \
                     `--no-runner` to keep it"
                    )
                })?;
            run_with = format!("`uv run {script}` (after `uv sync`)");
            changes.push(Change::write("pyproject.toml", pyproject));
        }
        Ok(Plan {
            changes,
            hint: format!(
                "Next: wire any `not yet wired` arms in {path}, script its test in {test_path}, \
                 fill in specs/{name}.md, then run it with {run_with}. See `amble help walk`."
            ),
        })
    }

    fn plan_ts(&self, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &["nodes", "edge"], &["no-runner"])
            .map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        let name = name.replace('-', "_");
        if !names::is_snake(&name) {
            return Err(format!(
                "`{name}` is not a walk name: use snake_case or kebab-case"
            ));
        }
        let nodes: Vec<String> = parsed
            .list("nodes")
            .into_iter()
            .map(|node| node.replace('-', "_"))
            .collect();
        if nodes.is_empty() {
            return Err(format!(
                "name the nodes to wire, starting node first\n{USAGE}"
            ));
        }
        let edges = parsed
            .list("edge")
            .into_iter()
            .map(parse_edge)
            .collect::<Result<Vec<_>, _>>()?;

        let target = Target::TypeScript;
        if !self.utils.exists("ambler.ts") {
            return Err(
                "ambler.ts not found: run this from the root of an Ambler project".to_string(),
            );
        }
        let path = target.walk_path(&name);
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        let missing: Vec<&String> = nodes
            .iter()
            .filter(|node| !self.utils.exists(&target.node_path(node)))
            .collect();
        if !missing.is_empty() {
            let commands: Vec<String> = missing
                .iter()
                .map(|node| format!("  amble node {node} (--outcomes <a,b> | --terminal)"))
                .collect();
            return Err(format!(
                "these nodes do not exist yet; create them first:\n{}",
                commands.join("\n")
            ));
        }
        let nodes = nodes
            .iter()
            .map(|node| {
                let path = target.node_path(node);
                ts::source::node_info(node, &self.read(&path)?).map_err(|e| format!("{path}: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let spec = ts::walk::WalkSpec::new(&name, nodes, edges);
        ts::walk::validate(&spec)?;
        let test_path = target.walk_test_path(&name).unwrap_or_default();
        let mut changes = vec![
            Change::source(target, &path, ts::walk::walk(&spec)?),
            Change::source(target, &test_path, ts::walk::tests(&spec)?),
        ];
        let spec_path = format!("specs/{name}.md");
        if !self.utils.exists(&spec_path) {
            changes.push(Change::write(spec_path, ts::walk::spec_md(&spec)?));
        }
        let mut run_with = format!("`bun {path}`");
        if !parsed.has("no-runner") {
            let package = self.read("package.json")?;
            let script = names::kebab(&name);
            let package = ts::project::add_script(&package, &name).ok_or_else(|| {
                format!(
                    "package.json already has a `{script}` script, or no `scripts`; pass \
                     `--no-runner` to keep it"
                )
            })?;
            run_with = format!("`bun run {script}`");
            changes.push(Change::write("package.json", package));
        }
        Ok(Plan {
            changes,
            hint: format!(
                "Next: wire any `unwired(…)` edges in {path}, script its test in {test_path}, \
                 fill in specs/{name}.md, then run it with {run_with}. See `amble help walk`."
            ),
        })
    }

    fn plan_rust(&self, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &["nodes", "edge"], &["no-runner"])
            .map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        if !names::is_snake(name) {
            return Err(format!("`{name}` is not a walk name: use snake_case"));
        }
        let nodes = parsed.list("nodes");
        if nodes.is_empty() {
            return Err(format!(
                "name the nodes to wire, starting node first\n{USAGE}"
            ));
        }
        let edges = parsed
            .list("edge")
            .into_iter()
            .map(parse_edge)
            .collect::<Result<Vec<_>, _>>()?;

        let path = format!("src/walks/{name}.rs");
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        let modules = register(&self.read(MODULES)?, name)
            .ok_or_else(|| format!("`{name}` is already registered in {MODULES}"))?;
        let traits = state_traits(&self.read(NODES)?).map_err(|e| format!("{NODES}: {e}"))?;

        let missing: Vec<&str> = nodes
            .iter()
            .copied()
            .filter(|node| !self.utils.exists(&format!("src/nodes/{node}.rs")))
            .collect();
        if !missing.is_empty() {
            let commands: Vec<String> = missing
                .iter()
                .map(|node| format!("  amble node {node} (--outcomes <a,b> | --terminal)"))
                .collect();
            return Err(format!(
                "these nodes do not exist yet; create them first:\n{}",
                commands.join("\n")
            ));
        }
        let nodes = nodes
            .into_iter()
            .map(|node| {
                let path = format!("src/nodes/{node}.rs");
                node_info(node, &self.read(&path)?).map_err(|e| format!("{path}: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let spec = WalkSpec {
            name: name.clone(),
            nodes,
            edges,
            traits,
        };
        validate(&spec)?;

        let mut changes = vec![
            Change::rust(path, walk(&spec)),
            Change::write(MODULES, modules),
        ];
        let spec_path = format!("specs/{name}.md");
        if !self.utils.exists(&spec_path) {
            changes.push(Change::write(spec_path, spec_md(&spec)));
        }
        let mut run_with = "its runner".to_string();
        if !parsed.has("no-runner") {
            let crate_name =
                crate_name(&self.read("Cargo.toml")?).ok_or("Cargo.toml has no package name")?;
            let main = "src/main.rs".to_string();
            let bin = format!("src/bin/{name}.rs");
            let runner_path = if self.utils.exists(&main) { bin } else { main };
            if self.utils.exists(&runner_path) {
                return Err(format!(
                    "{runner_path} already exists; pass `--no-runner` to keep it"
                ));
            }
            run_with = if runner_path == "src/main.rs" {
                "`cargo run`".to_string()
            } else {
                format!("`cargo run --bin {name}`")
            };
            changes.push(Change::write(runner_path, runner(&crate_name, name)));
        }
        Ok(Plan {
            changes,
            hint: format!(
                "Next: wire any `todo!` edges in src/walks/{name}.rs, script its test, fill in \
                 specs/{name}.md, then run it with {run_with}. See `amble help walk`."
            ),
        })
    }
}

impl<S, U> Node<S> for PlanWalk<U>
where
    S: HasTarget + HasArgs + HasPlan + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        Ok(match self.plan(state.target(), state.args()) {
            Ok(plan) => {
                *state.plan_mut() = plan;
                Outcome::Planned
            }
            Err(message) => {
                *state.message_mut() = Some(message);
                Outcome::Rejected
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Fake, State};

    const PROJECT: &[(&str, &str)] = &[
        ("Cargo.toml", "[package]\nname = \"my-app\"\n"),
        ("src/walks/mod.rs", "//! Walks.\n"),
        (
            "src/nodes/mod.rs",
            "pub mod count;\npub mod start;\npub mod stop;\n\npub trait HasCount {\n    \
             fn count(&self) -> i32;\n    fn count_mut(&mut self) -> &mut i32;\n}\n",
        ),
        (
            "src/nodes/start.rs",
            include_str!("../../templates/rust/example/start.rs"),
        ),
        (
            "src/nodes/count.rs",
            include_str!("../../templates/rust/example/count.rs"),
        ),
        (
            "src/nodes/stop.rs",
            include_str!("../../templates/rust/example/stop.rs"),
        ),
    ];
    const ARGS: &[&str] = &[
        "counter",
        "--nodes",
        "start,count,stop",
        "--edge",
        "start.Success=count",
    ];

    async fn run(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_args(args);
        let Ok(outcome) = PlanWalk::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    async fn rejection(fake: &Fake, args: &[&str]) -> String {
        let (outcome, state) = run(fake, args).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.plan.changes.is_empty());
        state.message.unwrap()
    }

    fn contents<'a>(state: &'a State, wanted: &str) -> &'a str {
        state
            .plan
            .changes
            .iter()
            .find_map(|change| match change {
                Change::Write { path, contents, .. } if path == wanted => Some(contents.as_str()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{wanted} is not planned"))
    }

    #[tokio::test]
    async fn plans_the_walk_its_registration_its_spec_and_the_main_runner() {
        let fake = Fake::with_files(PROJECT);

        let (outcome, state) = run(&fake, ARGS).await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(Change::path).collect();
        assert_eq!(
            paths,
            [
                "src/walks/counter.rs",
                MODULES,
                "specs/counter.md",
                "src/main.rs"
            ]
        );
        let walk = contents(&state, "src/walks/counter.rs");
        assert!(walk.contains("start::Outcome::Success => Transition::Next(Id::Count),"));
        assert!(walk.contains("count::Outcome::Again => todo"));
        assert_eq!(
            contents(&state, MODULES),
            "//! Walks.\n\npub mod counter;\n"
        );
        assert!(contents(&state, "specs/counter.md").contains("### Count"));
        assert!(contents(&state, "src/main.rs").contains("my_app::walks::counter::run()"));
        assert!(state.plan.hint.contains("`cargo run`"));
    }

    #[tokio::test]
    async fn a_second_walk_runs_from_src_bin_and_an_existing_spec_is_kept() {
        let mut files = PROJECT.to_vec();
        files.extend([
            ("src/main.rs", "fn main() {}"),
            ("specs/counter.md", "# Mine\n"),
        ]);
        let fake = Fake::with_files(&files);

        let (outcome, state) = run(&fake, ARGS).await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(Change::path).collect();
        assert_eq!(
            paths,
            ["src/walks/counter.rs", MODULES, "src/bin/counter.rs"]
        );
    }

    #[tokio::test]
    async fn no_runner_is_planned_when_asked_not_to() {
        let fake = Fake::with_files(PROJECT);
        let mut args = ARGS.to_vec();
        args.push("--no-runner");

        let (outcome, state) = run(&fake, &args).await;

        assert_eq!(outcome, Outcome::Planned);
        let paths: Vec<&str> = state.plan.changes.iter().map(Change::path).collect();
        assert_eq!(paths, ["src/walks/counter.rs", MODULES, "specs/counter.md"]);
    }

    const TS_PROJECT: &[(&str, &str)] = &[
        ("ambler.ts", ""),
        (
            "package.json",
            include_str!("../../templates/ts/package.json"),
        ),
        (
            "nodes/start.ts",
            include_str!("../../templates/ts/example/nodes/start.ts"),
        ),
        (
            "nodes/count.ts",
            include_str!("../../templates/ts/example/nodes/count.ts"),
        ),
        (
            "nodes/stop.ts",
            include_str!("../../templates/ts/example/nodes/stop.ts"),
        ),
    ];

    const PY: Target = Target::Python { package: "app" };

    /// A Python project, package `app`, with the example's nodes and no walk.
    fn py_project() -> Fake {
        let mut files = vec![
            (
                "pyproject.toml".to_string(),
                "[project]\nname = \"app\"\n\n[project.scripts]\n\n[build-system]\n".to_string(),
            ),
            ("src/app/engine.py".to_string(), String::new()),
        ];
        files.extend(
            crate::scaffold::py::project::example("app")
                .into_iter()
                .filter(|(path, _)| path.starts_with("src/app/nodes/")),
        );
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(path, contents)| (path.as_str(), contents.as_str()))
            .collect();
        Fake::with_files(&files)
    }

    async fn run_py(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_target(PY, args);
        let Ok(outcome) = PlanWalk::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn plans_a_python_walk_its_test_its_spec_and_its_script() {
        let fake = py_project();

        let (outcome, state) = run_py(
            &fake,
            &[
                "my-counter",
                "--nodes",
                "start,count,stop",
                "--edge",
                "start.success=count",
            ],
        )
        .await;

        assert_eq!(outcome, Outcome::Planned, "{:?}", state.message);
        let paths: Vec<&str> = state.plan.changes.iter().map(Change::path).collect();
        assert_eq!(
            paths,
            [
                "src/app/walks/my_counter.py",
                "tests/walks/test_my_counter.py",
                "specs/my_counter.md",
                "pyproject.toml"
            ]
        );
        let walk = contents(&state, "src/app/walks/my_counter.py");
        assert!(walk.contains("from app.engine import Walk\n"));
        assert!(walk.contains("class MyCounter(Walk[Id, State]):\n"));
        assert!(walk.contains("not yet wired: COUNT.FINISHED"));
        assert!(
            contents(&state, "pyproject.toml")
                .contains("[project.scripts]\nmy-counter = \"app.walks.my_counter:main\"\n")
        );
        assert!(state.plan.hint.contains("`uv run my-counter`"));

        let (outcome, state) = run_py(&fake, &["hello", "--nodes", "start,greet"]).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.message.unwrap().contains("amble node greet"));
    }

    async fn run_ts(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_target(Target::TypeScript, args);
        let Ok(outcome) = PlanWalk::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    #[tokio::test]
    async fn plans_a_typescript_walk_its_test_its_spec_and_its_script() {
        let fake = Fake::with_files(TS_PROJECT);

        let (outcome, state) = run_ts(
            &fake,
            &[
                "my-counter",
                "--nodes",
                "start,count,stop",
                "--edge",
                "start.success=count",
            ],
        )
        .await;

        assert_eq!(outcome, Outcome::Planned, "{:?}", state.message);
        let paths: Vec<&str> = state.plan.changes.iter().map(Change::path).collect();
        assert_eq!(
            paths,
            [
                "walks/my-counter.ts",
                "walks/tests/my-counter.test.ts",
                "specs/my_counter.md",
                "package.json"
            ]
        );
        let walk = contents(&state, "walks/my-counter.ts");
        assert!(walk.contains("onSuccess: \"COUNT\""));
        assert!(walk.contains("onStop: unwired(\"COUNT.onStop\")"));
        assert!(
            contents(&state, "package.json")
                .contains("\"my-counter\": \"bun walks/my-counter.ts\"")
        );
        assert!(state.plan.hint.contains("`bun run my-counter`"));
    }

    #[tokio::test]
    async fn rejects_what_a_typescript_project_does_not_allow() {
        let fake = Fake::with_files(TS_PROJECT);

        let (_, state) = run_ts(&fake, &["hello", "--nodes", "start,greet"]).await;
        assert!(state.message.unwrap().contains("amble node greet"));

        let (_, state) = run_ts(
            &fake,
            &["hello", "--nodes", "start", "--edge", "start.nope=start"],
        )
        .await;
        assert!(
            state
                .message
                .unwrap()
                .contains("no such outcome (it has onSuccess, onError)")
        );

        let (_, state) = run_ts(&Fake::default(), &["hello", "--nodes", "start"]).await;
        assert!(state.message.unwrap().contains("root of an Ambler project"));
    }

    #[tokio::test]
    async fn rejects_bad_arguments() {
        let fake = Fake::with_files(PROJECT);

        assert_eq!(rejection(&fake, &[]).await, USAGE);
        assert!(
            rejection(&fake, &["Counter", "--nodes", "start"])
                .await
                .contains("snake_case")
        );
        assert!(
            rejection(&fake, &["counter"])
                .await
                .contains("name the nodes to wire")
        );
        assert!(
            rejection(
                &fake,
                &["counter", "--nodes", "start", "--edge", "start=stop"]
            )
            .await
            .contains("should look like")
        );
        assert!(
            rejection(
                &fake,
                &["counter", "--nodes", "start", "--edge", "start.Nope=start"]
            )
            .await
            .contains("no such outcome")
        );
    }

    #[tokio::test]
    async fn names_the_nodes_that_have_to_be_created_first() {
        let fake = Fake::with_files(PROJECT);

        let message = rejection(&fake, &["counter", "--nodes", "start,ask,stop,tell"]).await;

        assert_eq!(
            message,
            "these nodes do not exist yet; create them first:\n  \
             amble node ask (--outcomes <a,b> | --terminal)\n  \
             amble node tell (--outcomes <a,b> | --terminal)"
        );
    }

    #[tokio::test]
    async fn rejects_what_the_project_does_not_allow() {
        let mut files = PROJECT.to_vec();
        files.push(("src/walks/counter.rs", ""));
        assert_eq!(
            rejection(&Fake::with_files(&files), ARGS).await,
            "src/walks/counter.rs already exists"
        );

        let mut files = PROJECT.to_vec();
        files.push(("src/nodes/broken.rs", "pub fn nothing() {}"));
        assert_eq!(
            rejection(&Fake::with_files(&files), &["counter", "--nodes", "broken"]).await,
            "src/nodes/broken.rs: it has no `impl Node`"
        );

        assert!(
            rejection(&Fake::default(), ARGS)
                .await
                .contains("root of an Ambler project")
        );
    }
}
