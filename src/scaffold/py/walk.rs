//! Generates a Python walk from the nodes it wires: `State`, `Id`, the walk class whose `step`
//! matches every outcome, `Error`, `run()` and `main()`, its wiring test and its spec.
//!
//! Like the nodes, the text is written the way ruff formats it.

use super::node::{bracket, field_default, from_import, imports, outcome_name};
use super::source::{Field, NodeInfo};
use crate::scaffold::names;
use crate::scaffold::walk::{self as shared, Edge, Wired};

/// Names a walk module binds itself: a node's module or class must not take them.
const RESERVED: &[&str] = &[
    "asyncio",
    "auto",
    "dataclass",
    "enum",
    "error",
    "field",
    "id",
    "main",
    "run",
    "state",
    "sys",
    "walk",
];

/// What `amble walk` was asked for, with what it read from the nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkSpec {
    /// The walk's name, `snake_case`.
    pub name: String,
    /// The import name of the project's package.
    pub package: String,
    /// The nodes to wire; the first is where the walk starts.
    pub nodes: Vec<NodeInfo>,
    /// The edges given, with each outcome spelled as an `Outcome` member: `INVALID_INPUT`.
    pub edges: Vec<Edge>,
}

impl WalkSpec {
    /// Builds the spec, spelling each edge's outcome the way the nodes do.
    #[must_use]
    pub fn new(name: &str, package: &str, nodes: Vec<NodeInfo>, edges: Vec<Edge>) -> Self {
        let edges = edges
            .into_iter()
            .map(|edge| Edge {
                outcome: outcome_name(&edge.outcome),
                ..edge
            })
            .collect();
        Self {
            name: name.to_string(),
            package: package.to_string(),
            nodes,
            edges,
        }
    }

    fn wired(&self) -> Vec<Wired> {
        self.nodes
            .iter()
            .map(|node| Wired {
                name: node.name.clone(),
                outcomes: node.outcomes.clone(),
            })
            .collect()
    }

    /// Every field of every node's state, each once, in the order the nodes name them.
    ///
    /// # Errors
    ///
    /// Returns a message naming a field two nodes give different types.
    pub fn state(&self) -> Result<Vec<Field>, String> {
        let mut fields: Vec<(Field, &str)> = Vec::new();
        for node in &self.nodes {
            for field in &node.state {
                match fields.iter().find(|(known, _)| known.name == field.name) {
                    Some((known, owner)) if known.ty != field.ty => {
                        return Err(format!(
                            "the nodes disagree on the state field `{}`: `{owner}` has `{}`, \
                             `{}` has `{}`",
                            field.name, known.ty, node.name, field.ty
                        ));
                    }
                    Some(_) => {}
                    None => fields.push((field.clone(), &node.name)),
                }
            }
        }
        Ok(fields.into_iter().map(|(field, _)| field).collect())
    }

    fn fallible(&self) -> Vec<&NodeInfo> {
        self.nodes.iter().filter(|node| node.fallible).collect()
    }
}

/// Checks that the nodes and edges make sense together, that the nodes agree on the state, and
/// that no node takes a name the walk module needs.
///
/// # Errors
///
/// Returns a message naming the first problem.
pub fn validate(spec: &WalkSpec) -> Result<(), String> {
    if let Some(node) = spec
        .nodes
        .iter()
        .find(|node| RESERVED.contains(&node.name.as_str()))
    {
        return Err(format!(
            "a walk module defines `{}` itself: rename the node `{}`",
            node.name, node.name
        ));
    }
    shared::validate(&spec.name, &spec.wired(), &spec.edges)?;
    spec.state().map(|_| ())
}

/// The indentation of what an outcome's `case` does.
const PAD: &str = "                        ";

/// `    <name>: <ty> = <default>`, with a factory call wrapped as ruff wraps it.
fn field_line(name: &str, ty: &str, default: &str) -> String {
    match default
        .strip_prefix("field(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        Some(argument) => bracket(
            4,
            &format!("{name}: {ty} = field("),
            &[argument.to_string()],
            ")",
        ),
        None => format!("    {name}: {ty} = {default}\n"),
    }
}

/// `READ_MOVE` for `read_move`.
fn id(name: &str) -> String {
    names::shout(name)
}

/// The `case` arms of one node in `step`.
fn arms(spec: &WalkSpec, node: &NodeInfo) -> String {
    let mut out = format!("            case Id.{}:\n", id(&node.name));
    let Some(outcomes) = &node.outcomes else {
        return out
            + &format!(
                "                await self.{}.run(state)\n                return None\n",
                node.name
            );
    };
    out += &bracket(
        16,
        &format!("match await self.{}.run(", node.name),
        &["state".to_string()],
        "):",
    );
    for outcome in outcomes {
        let edge = spec
            .edges
            .iter()
            .find(|edge| edge.from == node.name && edge.outcome == *outcome);
        let then = match edge {
            Some(Edge { to: Some(to), .. }) => format!("{PAD}return Id.{}\n", id(to)),
            Some(Edge { to: None, .. }) => format!("{PAD}return None\n"),
            None => bracket(
                PAD.len(),
                "raise NotImplementedError(",
                &[format!(
                    "\"{}: {}.{outcome}\"",
                    concat!("not yet ", "wired"),
                    id(&node.name)
                )],
                ")",
            ),
        };
        out += &format!(
            "                    case {}.Outcome.{outcome}:\n{then}",
            node.name
        );
    }
    out
}

/// The contents of `src/<package>/walks/<name>.py`.
///
/// # Errors
///
/// Returns a message if the nodes disagree on the state.
pub fn walk(spec: &WalkSpec) -> Result<String, String> {
    let state = spec.state()?;
    let fallible = spec.fallible();
    let pkg = &spec.package;

    let mut header = vec!["import asyncio".to_string()];
    if !fallible.is_empty() {
        header.push("import sys".to_string());
    }
    header.push("from dataclasses import dataclass, field".to_string());
    header.push("from enum import Enum, auto".to_string());
    header.push(format!("from {pkg}.engine import Walk"));
    let modules: Vec<&str> = spec
        .nodes
        .iter()
        .filter(|node| node.outcomes.is_some() || node.fallible)
        .map(|node| node.name.as_str())
        .collect();
    if !modules.is_empty() {
        header.push(from_import(&format!("{pkg}.nodes"), &modules));
    }
    for node in &spec.nodes {
        let class = names::pascal(&node.name);
        header.push(from_import(
            &format!("{pkg}.nodes.{}", node.name),
            &[&class],
        ));
        header.extend(node.imports.iter().cloned());
    }
    let mut out = format!(
        "\"\"\"The {} walk: see ``specs/{}.md``.\"\"\"\n\n",
        names::title(&spec.name).to_lowercase(),
        spec.name
    );
    out += &imports(pkg, &header);

    out += if fallible.is_empty() {
        "\n__all__ = [\"main\", \"run\"]\n"
    } else {
        "\n__all__ = [\"Error\", \"main\", \"run\"]\n"
    };
    match fallible.as_slice() {
        [] => {}
        [one] => out += &format!("\nError = {}.Error\n", one.name),
        many => {
            let errors: Vec<String> = many
                .iter()
                .map(|node| format!("{}.Error", node.name))
                .collect();
            out += "\n";
            out += &bracket(0, "Error = (", &errors, ")");
        }
    }
    if !fallible.is_empty() {
        out += "\"\"\"What can stop the walk early.\"\"\"\n";
    }

    out += "\n\n@dataclass\nclass State:\n";
    if state.is_empty() {
        out += "    \"\"\"The walk's nodes share no state.\"\"\"\n";
    }
    for field in &state {
        out += &field_line(&field.name, &field.ty, &field_default(&field.ty));
    }

    out += "\n\nclass Id(Enum):\n";
    for node in &spec.nodes {
        out += &format!("    {} = auto()\n", id(&node.name));
    }

    let class = names::pascal(&spec.name);
    out += &format!("\n\n@dataclass(frozen=True)\nclass {class}(Walk[Id, State]):\n");
    for node in &spec.nodes {
        let node_class = names::pascal(&node.name);
        out += &field_line(
            &node.name,
            &node_class,
            &format!("field(default_factory={node_class})"),
        );
    }
    out += "\n    async def step(self, id: Id, state: State) -> Id | None:\n        match id:\n";
    for node in &spec.nodes {
        out += &arms(spec, node);
    }

    let start = id(&spec.nodes[0].name);
    out += &format!(
        "\n\nasync def run() -> None:\n    \"\"\"Runs the walk from its first node.\"\"\"\n    \
         await {class}().run(Id.{start}, State())\n"
    );
    let runs = if fallible.is_empty() {
        "Runs the walk"
    } else {
        "Runs the walk and reports an error"
    };
    let doc = format!(
        "    \"\"\"{runs}: the ``{}`` script in pyproject.toml.\"\"\"\n",
        names::kebab(&spec.name)
    );
    let doc = if doc.len() <= super::node::WIDTH + 1 {
        doc
    } else {
        "    \"\"\"Runs the walk: its script in pyproject.toml.\"\"\"\n".to_string()
    };
    out += &format!("\n\ndef main() -> None:\n{doc}");
    if fallible.is_empty() {
        out += "    asyncio.run(run())\n";
    } else {
        out += "    try:\n        asyncio.run(run())\n    except Error as e:\n        \
                sys.exit(f\"Error: {e}\")\n";
    }
    Ok(out)
}

/// The contents of `tests/walks/test_<name>.py`: a script to fill in and one test to rename.
///
/// # Errors
///
/// Returns a message if the nodes disagree on the state.
pub fn tests(spec: &WalkSpec) -> Result<String, String> {
    spec.state()?;
    let pkg = &spec.package;
    let class = names::pascal(&spec.name);
    let start = id(&spec.nodes[0].name);
    let scripted = spec.nodes.iter().any(|node| node.utils);
    let mut header = Vec::new();
    if scripted {
        for node in &spec.nodes {
            header.push(format!(
                "from {pkg}.nodes.{} import {}",
                node.name,
                names::pascal(&node.name)
            ));
        }
    }
    header.push(from_import(
        &format!("{pkg}.walks.{}", spec.name),
        &[class.as_str(), "Id", "State"],
    ));
    let mut out = imports(pkg, &header);
    let walk = if scripted {
        out += "\n\nclass Script:\n    \
                \"\"\"Plays back scripted inputs and records what the nodes do.\n\n    \
                Implements the ``Utils`` protocol of every node in the walk.\n    \"\"\"\n\n    \
                def __init__(self) -> None:\n        \
                raise NotImplementedError(\"not yet implemented\")\n";
        let nodes: Vec<String> = spec
            .nodes
            .iter()
            .map(|node| format!("{}={}(script)", node.name, names::pascal(&node.name)))
            .collect();
        out += &format!("\n\ndef {}(script: Script) -> {class}:\n", spec.name);
        out += &bracket(4, &format!("return {class}("), &nodes, ")");
        format!("{}(script)", spec.name)
    } else {
        format!("{class}()")
    };
    out += &format!(
        "\n\nasync def test_{name}_walks_from_start_to_halt() -> None:\n    \
         \"\"\"One test per path through the walk: rename this after the path it follows.\"\"\"\n",
        name = spec.name
    );
    if scripted {
        out += "    script = Script()\n";
    }
    out += &format!(
        "    state = State()\n\n    await {walk}.run(Id.{start}, state)\n\n    \
         # TODO: assert the final state and what the script recorded.\n    \
         assert state == State()\n"
    );
    Ok(out)
}

/// The contents of `specs/<name>.md`.
///
/// # Errors
///
/// Returns a message if the nodes disagree on the state.
pub fn spec_md(spec: &WalkSpec) -> Result<String, String> {
    let fields: Vec<String> = spec.state()?.into_iter().map(|field| field.name).collect();
    Ok(shared::spec_md(&spec.wired(), &spec.edges, &fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::py::project::example;
    use crate::scaffold::py::source::{node_info, parse};
    use crate::scaffold::walk::parse_edge;

    /// An example file, in the package `app`.
    fn file(path: &str) -> String {
        example("app")
            .into_iter()
            .find(|(known, _)| known == path)
            .unwrap_or_else(|| panic!("no {path} in the example"))
            .1
    }

    fn counter(edges: &[&str]) -> WalkSpec {
        let init = parse(&file("src/app/nodes/__init__.py")).unwrap();
        let node = |name: &str| {
            node_info(name, &file(&format!("src/app/nodes/{name}.py")), &init).unwrap()
        };
        WalkSpec::new(
            "counter",
            "app",
            vec![node("start"), node("count"), node("stop")],
            edges.iter().map(|edge| parse_edge(edge).unwrap()).collect(),
        )
    }

    const EDGES: &[&str] = &[
        "start.success=count",
        "start.InvalidInput=start",
        "count.again=count",
        "count.finished=stop",
    ];

    #[test]
    fn reproduces_the_reference_counter_walk() {
        let spec = counter(EDGES);

        assert_eq!(validate(&spec), Ok(()));
        assert_eq!(walk(&spec).unwrap(), file("src/app/walks/counter.py"));
    }

    #[test]
    fn leaves_edges_it_was_not_given_unwired() {
        let generated = walk(&counter(&["start.success=count"])).unwrap();

        assert!(generated.contains(
            "                    case start.Outcome.INVALID_INPUT:\n                        raise \
             NotImplementedError(\"not yet wired: START.INVALID_INPUT\")\n"
        ));
        assert!(generated.contains("not yet wired: COUNT.FINISHED"));
        assert!(parse(&generated).is_ok());
    }

    #[test]
    fn a_walk_that_cannot_fail_has_no_error() {
        let mut spec = counter(EDGES);
        spec.nodes[0].fallible = false;

        let generated = walk(&spec).unwrap();

        assert!(!generated.contains("Error"));
        assert!(!generated.contains("import sys"));
        assert!(generated.contains("__all__ = [\"main\", \"run\"]\n"));
        assert!(generated.ends_with("    asyncio.run(run())\n"));

        spec.nodes[0].fallible = true;
        spec.nodes[1].fallible = true;
        assert!(
            walk(&spec)
                .unwrap()
                .contains("\nError = (start.Error, count.Error)\n")
        );
    }

    #[test]
    fn checks_the_edges_the_state_and_the_names() {
        let error = |edges: &[&str]| validate(&counter(edges)).unwrap_err();
        assert!(
            error(&["start.nope=count"])
                .contains("`start` has no such outcome (it has SUCCESS, INVALID_INPUT)")
        );
        assert!(error(&["stop.done=start"]).contains("terminal node"));

        let mut spec = counter(EDGES);
        spec.nodes[2].state[0].ty = "str".to_string();
        assert_eq!(
            validate(&spec).unwrap_err(),
            "the nodes disagree on the state field `count`: `start` has `int`, `stop` has `str`"
        );

        let mut spec = counter(EDGES);
        spec.nodes[2].name = "run".to_string();
        assert!(
            validate(&spec)
                .unwrap_err()
                .contains("rename the node `run`")
        );
    }

    #[test]
    fn writes_a_test_to_finish_and_a_spec() {
        let spec = counter(EDGES);

        let tests = tests(&spec).unwrap();
        assert!(tests.starts_with("from app.nodes.count import Count\n"));
        assert!(tests.contains("from app.walks.counter import Counter, Id, State\n"));
        assert!(tests.contains("raise NotImplementedError(\"not yet implemented\")"));
        assert!(tests.contains(
            "    return Counter(start=Start(script), count=Count(script), stop=Stop(script))\n"
        ));
        assert!(tests.contains("    await counter(script).run(Id.START, state)\n"));
        assert!(parse(&tests).is_ok());

        let md = spec_md(&spec).unwrap();
        assert!(md.contains("- TODO: when `AGAIN`, it transitions to `COUNT`.\n"));
        assert!(
            md.contains(
                "### Stop\n- TODO: describe what this node does.\n- Terminates the walk.\n"
            )
        );
    }
}
