use crate::Node;
use crate::nodes::{HasArgs, HasMessage, HasPlan, HasTarget};
use crate::scaffold::rust::node::{NodeSpec, has_trait, node, shorten, trait_name};
use crate::scaffold::rust::register::{add_use, append, register};
use crate::scaffold::rust::source::state_traits;
use crate::scaffold::target::Target;
use crate::scaffold::ts::{self, source::Field};
use crate::scaffold::{Change, Plan, args, names};
use crate::utils::fs;
use std::convert::Infallible;
use std::io;

const USAGE: &str = "usage: amble node <name> (--outcomes <a,b> | --terminal) [--fallible] \
                     [--state <field>[:<Type>]]...";
const MODULES: &str = "src/nodes/mod.rs";

/// Where [`PlanNode`] leaves the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The plan holds the node file and its registration.
    Planned,
    /// The arguments or the project do not allow it; the message says why.
    Rejected,
}

pub trait Utils {
    fn exists(&self, path: &str) -> bool;
    fn read(&self, path: &str) -> io::Result<String>;
    /// The names of the entries in the folder at `path`.
    fn entries(&self, path: &str) -> io::Result<Vec<String>>;
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

    fn entries(&self, path: &str) -> io::Result<Vec<String>> {
        fs::entries(path)
    }
}

/// Plans a new node. In Rust: its skeleton in `src/nodes/`, its registration and any new state
/// trait. In TypeScript: its skeleton in `nodes/` and its failing tests in `nodes/tests/`.
#[derive(Clone, Debug, Default)]
pub struct PlanNode<U = SystemUtils> {
    utils: U,
}

impl PlanNode {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<U> PlanNode<U> {
    #[must_use]
    pub fn with_utils(utils: U) -> Self {
        Self { utils }
    }
}

impl<U: Utils> PlanNode<U> {
    fn plan(&self, target: Target, args: &[String]) -> Result<Plan, String> {
        match target {
            Target::Rust => self.plan_rust(args),
            Target::TypeScript => self.plan_ts(args),
        }
    }

    fn plan_rust(&self, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &["outcomes", "state"], &["terminal", "fallible"])
            .map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        if !names::is_snake(name) {
            return Err(format!("`{name}` is not a node name: use snake_case"));
        }
        let outcomes: Vec<String> = parsed
            .list("outcomes")
            .into_iter()
            .map(names::pascal)
            .collect();
        if outcomes.is_empty() != parsed.has("terminal") {
            return Err(format!(
                "say what the node reports: `--outcomes <a,b>`, or `--terminal` if it ends the \
                 walk\n{USAGE}"
            ));
        }
        if let Some(outcome) = outcomes.iter().find(|outcome| !names::is_pascal(outcome)) {
            return Err(format!("`{outcome}` is not an outcome name"));
        }

        let path = format!("src/nodes/{name}.rs");
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }
        let modules = self.utils.read(MODULES).map_err(|_| {
            format!("{MODULES} not found: run this from the root of an Ambler project")
        })?;
        let mut modules = register(&modules, name)
            .ok_or_else(|| format!("`{name}` is already registered in {MODULES}"))?;

        // `--state field` reuses a trait; `--state field:Type` declares a new one.
        let known = state_traits(&modules).map_err(|e| format!("{MODULES}: {e}"))?;
        let mut traits = Vec::new();
        for state in parsed.values("state") {
            let (field, ty) = match state.split_once(':') {
                Some((field, ty)) => (field.trim(), Some(ty.trim())),
                None => (state.trim(), None),
            };
            if !names::is_snake(field) {
                return Err(format!("`{field}` is not a field name: use snake_case"));
            }
            let wanted = trait_name(field);
            let existing = known.iter().find(|item| item.name == wanted);
            match (existing, ty) {
                (Some(item), None) => traits.push(item.clone()),
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "`{wanted}` is already in {MODULES}: use `--state {field}`"
                    ));
                }
                (None, None) => {
                    return Err(format!(
                        "no `{wanted}` in {MODULES}: give the type, `--state {field}:<Type>`"
                    ));
                }
                (None, Some(ty)) => {
                    // `crate::board::Board` is declared as `Board`, imported in mod.rs.
                    let (short, imports) = shorten(ty);
                    let declaration = has_trait(field, &short);
                    state_traits(&declaration).map_err(|_| format!("`{ty}` is not a type"))?;
                    for import in imports {
                        modules = add_use(&modules, &import);
                    }
                    modules = append(&modules, &declaration);
                    let declared = state_traits(&modules)
                        .map_err(|e| format!("{MODULES}: {e}"))?
                        .into_iter()
                        .find(|item| item.name == wanted);
                    traits.extend(declared);
                }
            }
        }

        let spec = NodeSpec {
            name: name.clone(),
            outcomes,
            fallible: parsed.has("fallible"),
            traits,
        };
        Ok(Plan {
            changes: vec![
                Change::rust(path, node(&spec)),
                Change::write(MODULES, modules),
            ],
            hint: format!(
                "Next: give `Utils` a method per side effect and write the tests, see them fail \
                 with `cargo test nodes::{name}::`, then implement `run`. See `amble help \
                 node-design`."
            ),
        })
    }
}

impl<U: Utils> PlanNode<U> {
    /// The type other nodes give the `State` field `field`, if one of them has it.
    fn known_field(&self, field: &str) -> Option<String> {
        let mut files = self.utils.entries("nodes").ok()?;
        files.sort();
        files
            .iter()
            .filter(|file| file.ends_with(".ts"))
            .filter_map(|file| self.utils.read(&format!("nodes/{file}")).ok())
            .filter_map(|source| ts::source::parse(&source).ok())
            .find_map(|module| {
                module
                    .object("State")?
                    .iter()
                    .find(|known| known.name == field)
                    .map(|known| known.ty.clone())
            })
    }

    fn plan_ts(&self, args: &[String]) -> Result<Plan, String> {
        let parsed = args::parse(args, &["outcomes", "state"], &["terminal", "fallible"])
            .map_err(|e| format!("{e}\n{USAGE}"))?;
        let [name] = parsed.positionals.as_slice() else {
            return Err(USAGE.to_string());
        };
        let name = name.replace('-', "_");
        if !names::is_snake(&name) {
            return Err(format!(
                "`{name}` is not a node name: use snake_case or kebab-case"
            ));
        }
        if parsed.has("fallible") {
            return Err(
                "`--fallible` is for Rust nodes: a TypeScript node throws when it cannot go on, \
                 and routes an expected failure through an edge such as `--outcomes ok,error`"
                    .to_string(),
            );
        }
        let terminal = parsed.has("terminal");
        let outcomes = parsed.list("outcomes");
        if outcomes.is_empty() != terminal {
            return Err(format!(
                "say what the node reports: `--outcomes <a,b>`, or `--terminal` if it ends the \
                 walk\n{USAGE}"
            ));
        }
        let edges: Vec<String> = if terminal {
            vec![ts::node::TERMINAL_EDGE.to_string()]
        } else {
            outcomes.into_iter().map(ts::node::edge_name).collect()
        };
        if let Some(edge) = edges.iter().find(|edge| !ts::node::is_edge_name(edge)) {
            return Err(format!("`{edge}` is not an edge name"));
        }

        if !self.utils.exists("ambler.ts") {
            return Err(
                "ambler.ts not found: run this from the root of an Ambler project".to_string(),
            );
        }
        let path = Target::TypeScript.node_path(&name);
        let test_path = Target::TypeScript.node_test_path(&name).unwrap_or_default();
        if self.utils.exists(&path) {
            return Err(format!("{path} already exists"));
        }

        // `--state field` takes the type another node gives it; `--state field:Type` sets it.
        let mut state: Vec<Field> = Vec::new();
        for given in parsed.values("state") {
            let (field, ty) = match given.split_once(':') {
                Some((field, ty)) => (field.trim(), Some(ty.trim())),
                None => (given.trim(), None),
            };
            let snake = names::snake(field);
            if !names::is_snake(&snake) {
                return Err(format!("`{field}` is not a field name"));
            }
            let field = names::camel(&snake);
            let ty = match ty {
                Some(ty) if !ty.is_empty() => ty.to_string(),
                _ => self.known_field(&field).ok_or_else(|| {
                    format!(
                        "no node has a `{field}` field yet: give the type, `--state \
                         {field}:<Type>`"
                    )
                })?,
            };
            if state.iter().all(|known| known.name != field) {
                state.push(Field {
                    name: field,
                    ty,
                    optional: false,
                });
            }
        }

        let spec = ts::node::NodeSpec {
            name: name.clone(),
            edges,
            terminal,
            state,
        };
        let declared = ts::node::node(&spec);
        ts::source::parse(&declared).map_err(|(_, message)| {
            format!("the generated node does not parse ({message}): check the `--state` types")
        })?;
        Ok(Plan {
            changes: vec![
                Change::source(Target::TypeScript, path, declared),
                Change::source(
                    Target::TypeScript,
                    test_path.clone(),
                    ts::node::tests(&spec),
                ),
            ],
            hint: format!(
                "Next: give `Utils` a function per side effect and write the tests in \
                 {test_path}, see them fail with `{}`, then implement the factory. See `amble \
                 help node-design`.",
                Target::TypeScript.node_test_command(&name)
            ),
        })
    }
}

impl<S, U> Node<S> for PlanNode<U>
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

    const PROJECT: &[(&str, &str)] = &[(
        "src/nodes/mod.rs",
        "//! Nodes.\n\npub mod stop;\n\n/// A state that has a count field.\npub trait HasCount \
         {\n    fn count(&self) -> i32;\n    fn count_mut(&mut self) -> &mut i32;\n}\n",
    )];

    async fn run(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_args(args);
        let Ok(outcome) = PlanNode::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    async fn rejection(fake: &Fake, args: &[&str]) -> String {
        let (outcome, state) = run(fake, args).await;
        assert_eq!(outcome, Outcome::Rejected);
        assert!(state.plan.changes.is_empty());
        state.message.unwrap()
    }

    #[tokio::test]
    async fn plans_the_node_file_and_registers_it_in_order() {
        let fake = Fake::with_files(PROJECT);

        let (outcome, state) = run(&fake, &["ask", "--outcomes", "success,invalid_input"]).await;

        assert_eq!(outcome, Outcome::Planned);
        let [
            Change::Write {
                path,
                contents,
                format: Some(crate::scaffold::target::Target::Rust),
            },
            modules,
        ] = state.plan.changes.as_slice()
        else {
            panic!(
                "expected the node file, then mod.rs: {:?}",
                state.plan.changes
            );
        };
        assert_eq!(path, "src/nodes/ask.rs");
        assert!(contents.contains("pub struct Ask<U = SystemUtils>"));
        assert!(contents.contains("    InvalidInput,\n"));
        assert_eq!(
            modules,
            &Change::write(
                MODULES,
                PROJECT[0]
                    .1
                    .replace("pub mod stop;", "pub mod ask;\npub mod stop;")
            )
        );
        assert!(state.plan.hint.contains("cargo test nodes::ask::"));
    }

    #[tokio::test]
    async fn reuses_a_state_trait_and_declares_a_new_one() {
        let fake = Fake::with_files(PROJECT);

        let (outcome, state) = run(
            &fake,
            &[
                "greet",
                "--terminal",
                "--state",
                "count",
                "--state",
                "name:String",
            ],
        )
        .await;

        assert_eq!(outcome, Outcome::Planned);
        let [
            Change::Write { contents: node, .. },
            Change::Write {
                contents: modules, ..
            },
        ] = state.plan.changes.as_slice()
        else {
            panic!("expected two files: {:?}", state.plan.changes);
        };
        assert!(node.contains("    S: HasCount + HasName + Send,\n"));
        assert!(modules.ends_with(
            "/// A state that has a name field.\npub trait HasName {\n    \
             fn name(&self) -> &String;\n    fn name_mut(&mut self) -> &mut String;\n}\n"
        ));
        assert_eq!(modules.matches("pub trait HasCount").count(), 1);
    }

    #[tokio::test]
    async fn imports_a_type_given_by_its_path_where_the_trait_and_the_tests_need_it() {
        let fake = Fake::with_files(PROJECT);

        let (outcome, state) = run(
            &fake,
            &[
                "place",
                "--terminal",
                "--state",
                "board:crate::board::Board",
            ],
        )
        .await;

        assert_eq!(outcome, Outcome::Planned);
        let [
            Change::Write { contents: node, .. },
            Change::Write {
                contents: modules, ..
            },
        ] = state.plan.changes.as_slice()
        else {
            panic!("expected two files: {:?}", state.plan.changes);
        };
        assert!(modules.contains("pub mod stop;\n\nuse crate::board::Board;\n\n"));
        assert!(modules.contains("    fn board(&self) -> &Board;\n"));
        assert!(node.contains("    use super::*;\n    use crate::board::Board;\n"));

        // Reusing the trait later imports the type for the tests again.
        let fake = Fake::with_files(&[("src/nodes/mod.rs", modules.as_str())]);
        let (_, state) = run(&fake, &["clear", "--terminal", "--state", "board"]).await;
        let Change::Write { contents: node, .. } = &state.plan.changes[0] else {
            panic!("expected the node file");
        };
        assert!(node.contains("    use crate::board::Board;\n"));
    }

    async fn run_ts(fake: &Fake, args: &[&str]) -> (Outcome, State) {
        let mut state = State::with_target(Target::TypeScript, args);
        let Ok(outcome) = PlanNode::with_utils(fake).run(&mut state).await;
        (outcome, state)
    }

    const TS_PROJECT: &[(&str, &str)] = &[
        ("ambler.ts", ""),
        (
            "nodes/count.ts",
            include_str!("../../templates/ts/example/nodes/count.ts"),
        ),
    ];

    #[tokio::test]
    async fn plans_a_typescript_node_and_its_tests() {
        let fake = Fake::with_files(TS_PROJECT);

        let (outcome, state) = run_ts(
            &fake,
            &[
                "read-move",
                "--outcomes",
                "valid,invalid_input",
                "--state",
                "count",
                "--state",
                "player_name:string",
            ],
        )
        .await;

        assert_eq!(outcome, Outcome::Planned, "{:?}", state.message);
        let [
            Change::Write {
                path,
                contents,
                format: Some(Target::TypeScript),
            },
            Change::Write {
                path: test_path,
                contents: tests,
                ..
            },
        ] = state.plan.changes.as_slice()
        else {
            panic!("expected the node and its tests: {:?}", state.plan.changes);
        };
        assert_eq!(path, "nodes/read-move.ts");
        assert!(contents.contains("export type Edge = \"onValid\" | \"onInvalidInput\";\n"));
        assert!(contents.contains("  count: number;\n  playerName: string;\n"));
        assert_eq!(test_path, "nodes/tests/read-move.test.ts");
        assert!(tests.contains("test(\"readMove takes onInvalidInput"));
        assert!(
            state
                .plan
                .hint
                .contains("`bun test nodes/tests/read-move.test.ts`")
        );
    }

    #[tokio::test]
    async fn plans_a_terminal_typescript_node() {
        let fake = Fake::with_files(TS_PROJECT);

        let (outcome, state) = run_ts(&fake, &["stop", "--terminal"]).await;

        assert_eq!(outcome, Outcome::Planned);
        let Change::Write { contents, .. } = &state.plan.changes[0] else {
            panic!("expected the node file");
        };
        assert!(contents.contains("export type Edge = \"onDone\";\n"));
    }

    #[tokio::test]
    async fn rejects_what_a_typescript_project_does_not_allow() {
        let fake = Fake::with_files(TS_PROJECT);
        let rejected = |args: &'static [&'static str]| {
            let fake = &fake;
            async move {
                let (outcome, state) = run_ts(fake, args).await;
                assert_eq!(outcome, Outcome::Rejected);
                state.message.unwrap()
            }
        };

        assert!(
            rejected(&["ask", "--terminal", "--fallible"])
                .await
                .contains("`--fallible` is for Rust nodes")
        );
        assert!(
            rejected(&["ask", "--terminal", "--state", "total"])
                .await
                .contains("no node has a `total` field yet")
        );
        assert_eq!(
            rejected(&["count", "--terminal"]).await,
            "nodes/count.ts already exists"
        );
        let (_, state) = run_ts(&Fake::default(), &["ask", "--terminal"]).await;
        assert!(state.message.unwrap().contains("root of an Ambler project"));
    }

    #[tokio::test]
    async fn rejects_bad_arguments() {
        let fake = Fake::with_files(PROJECT);

        assert_eq!(rejection(&fake, &[]).await, USAGE);
        assert!(
            rejection(&fake, &["Ask", "--terminal"])
                .await
                .contains("snake_case")
        );
        assert!(
            rejection(&fake, &["ask"])
                .await
                .contains("say what the node reports")
        );
        assert!(
            rejection(&fake, &["ask", "--terminal", "--outcomes", "a"])
                .await
                .contains("say what the node reports")
        );
        assert!(
            rejection(&fake, &["ask", "--nope"])
                .await
                .contains("unknown flag `--nope`")
        );
    }

    #[tokio::test]
    async fn rejects_what_the_project_does_not_allow() {
        let fake = Fake::with_files(PROJECT);
        assert!(
            rejection(&fake, &["stop", "--terminal"])
                .await
                .contains("already registered")
        );
        assert!(
            rejection(&fake, &["ask", "--terminal", "--state", "total"])
                .await
                .contains("no `HasTotal` in src/nodes/mod.rs")
        );
        assert!(
            rejection(&fake, &["ask", "--terminal", "--state", "count:i64"])
                .await
                .contains("use `--state count`")
        );

        let taken = Fake::with_files(&[PROJECT[0], ("src/nodes/ask.rs", "")]);
        assert_eq!(
            rejection(&taken, &["ask", "--terminal"]).await,
            "src/nodes/ask.rs already exists"
        );

        let nowhere = Fake::default();
        assert!(
            rejection(&nowhere, &["ask", "--terminal"])
                .await
                .contains("root of an Ambler project")
        );
    }
}
