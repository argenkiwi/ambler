use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan};
use crate::scaffold::project::crate_name;
use crate::scaffold::register::register;
use crate::scaffold::source::{node_info, state_traits};
use crate::scaffold::walk::{WalkSpec, parse_edge, runner, spec_md, validate, walk};
use crate::scaffold::{Change, Plan, args, names};
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

    fn plan(&self, args: &[String]) -> Result<Plan, String> {
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
    S: HasArgs + HasPlan + HasMessage + Send,
    U: Utils + Sync,
{
    type Outcome = Outcome;
    type Error = Infallible;

    async fn run(&self, state: &mut S) -> Result<Outcome, Infallible> {
        Ok(match self.plan(state.args()) {
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
            include_str!("../../templates/example/start.rs"),
        ),
        (
            "src/nodes/count.rs",
            include_str!("../../templates/example/count.rs"),
        ),
        (
            "src/nodes/stop.rs",
            include_str!("../../templates/example/stop.rs"),
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
