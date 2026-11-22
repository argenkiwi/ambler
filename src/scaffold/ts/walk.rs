//! Generates a TypeScript walk from the nodes it wires: `State`, `NodeId`, `WalkUtils`,
//! `createWalk`, the `import.meta.main` runner, its wiring test and its spec.
//!
//! Like the nodes, the text is written the way Biome formats it.

use super::node::{TERMINAL_EDGE, default_value, edge_name, literal, object_literal, union};
use super::source::{Field, NodeInfo};
use crate::scaffold::names;
use crate::scaffold::walk::{self as shared, Edge, Wired};

const WIDTH: usize = 80;

/// What `amble walk` was asked for, with what it read from the nodes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkSpec {
    /// The walk's name, `snake_case`.
    pub name: String,
    /// The nodes to wire; the first is where the walk starts.
    pub nodes: Vec<NodeInfo>,
    /// The edges given, with each outcome spelled as an edge name: `onSuccess`.
    pub edges: Vec<Edge>,
}

impl WalkSpec {
    /// Builds the spec, spelling each edge's outcome the way the nodes do.
    #[must_use]
    pub fn new(name: &str, nodes: Vec<NodeInfo>, edges: Vec<Edge>) -> Self {
        let edges = edges
            .into_iter()
            .map(|edge| Edge {
                outcome: edge_name(&edge.outcome),
                ..edge
            })
            .collect();
        Self {
            name: name.to_string(),
            nodes,
            edges,
        }
    }

    /// Whether `node` ends the walk: its one edge is `onDone` and no edge was given for it.
    fn terminal(&self, node: &NodeInfo) -> bool {
        node.edges == [TERMINAL_EDGE] && self.edges.iter().all(|edge| edge.from != node.name)
    }

    fn wired(&self) -> Vec<Wired> {
        self.nodes
            .iter()
            .map(|node| Wired {
                name: node.name.clone(),
                outcomes: (!self.terminal(node)).then(|| node.edges.clone()),
            })
            .collect()
    }

    /// Every field of every node's `State`, each once, in the order the nodes name them.
    ///
    /// # Errors
    ///
    /// Returns a message naming a field two nodes give different types.
    pub fn state(&self) -> Result<Vec<Field>, String> {
        let mut fields: Vec<(Field, &str)> = Vec::new();
        for node in &self.nodes {
            for field in &node.state {
                match fields
                    .iter_mut()
                    .find(|(known, _)| known.name == field.name)
                {
                    Some((known, owner)) if known.ty != field.ty => {
                        return Err(format!(
                            "the nodes disagree on the state field `{}`: `{owner}` has `{}`, \
                             `{}` has `{}`",
                            field.name, known.ty, node.name, field.ty
                        ));
                    }
                    Some((known, _)) => known.optional &= field.optional,
                    None => fields.push((field.clone(), &node.name)),
                }
            }
        }
        Ok(fields.into_iter().map(|(field, _)| field).collect())
    }
}

/// Checks that the nodes and edges make sense together, and that the nodes agree on the state.
///
/// # Errors
///
/// Returns a message naming the first problem.
pub fn validate(spec: &WalkSpec) -> Result<(), String> {
    shared::validate(&spec.name, &spec.wired(), &spec.edges)?;
    spec.state().map(|_| ())
}

/// The id of a node in the walk: `read_move` → `READ_MOVE`.
fn id(node: &str) -> String {
    names::shout(node)
}

/// What an edge is wired to: a quoted id, `null`, or `unwired(...)` if it was not given.
fn target(spec: &WalkSpec, node: &NodeInfo, edge: &str) -> String {
    if spec.terminal(node) {
        return "null".to_string();
    }
    match spec
        .edges
        .iter()
        .find(|given| given.from == node.name && given.outcome == edge)
    {
        Some(Edge { to: Some(to), .. }) => format!("\"{}\"", id(to)),
        Some(Edge { to: None, .. }) => "null".to_string(),
        None => format!("unwired(\"{}.{edge}\")", id(&node.name)),
    }
}

/// `    KEY: () => node.factory({ … }, utils.node),` as Biome breaks it.
fn entry(spec: &WalkSpec, node: &NodeInfo) -> String {
    let camel = names::camel(&node.name);
    let edges: Vec<String> = node
        .edges
        .iter()
        .map(|edge| format!("{edge}: {}", target(spec, node, edge)))
        .collect();
    let utils = if node.utils {
        format!(", utils.{camel}")
    } else {
        String::new()
    };
    let call = format!("{camel}Node.factory({{ {} }}{utils})", edges.join(", "));
    let flat = format!("    {}: () => {call},\n", id(&node.name));
    if flat.len() <= WIDTH + 1 {
        return flat;
    }
    let broken = format!("      {call},\n");
    if broken.len() <= WIDTH + 1 {
        return format!("    {}: () =>\n{broken}", id(&node.name));
    }
    let mut out = format!(
        "    {}: () =>\n      {camel}Node.factory(\n        {{\n",
        id(&node.name)
    );
    for edge in &edges {
        out += &format!("          {edge},\n");
    }
    out += "        },\n";
    if node.utils {
        out += &format!("        utils.{camel},\n");
    }
    out + "      ),\n"
}

fn unfinished(spec: &WalkSpec, state: &[Field]) -> bool {
    let unwired_edge = spec.nodes.iter().any(|node| {
        node.edges
            .iter()
            .any(|edge| target(spec, node, edge).starts_with("unwired"))
    });
    let unknown_field = state
        .iter()
        .any(|field| !field.optional && default_value(&field.ty).is_none());
    unwired_edge || unknown_field
}

/// The value a walk starts from, with `unwired(...)` for fields that have no obvious one.
fn initial(state: &[Field], indent: usize, prefix: &str, suffix: &str) -> String {
    let properties: Vec<String> = state
        .iter()
        .filter(|field| !field.optional)
        .map(|field| {
            let value = default_value(&field.ty)
                .unwrap_or_else(|| format!("unwired(\"initial {}\")", field.name));
            format!("{}: {value}", field.name)
        })
        .collect();
    literal(&properties, indent, prefix, suffix)
}

/// The contents of `walks/<name>.ts`.
///
/// # Errors
///
/// Returns a message if the nodes disagree on the state.
pub fn walk(spec: &WalkSpec) -> Result<String, String> {
    let state = spec.state()?;
    let mut out = String::from("import { ambler, walk } from \"../ambler.ts\";\n");
    let mut sorted: Vec<&NodeInfo> = spec.nodes.iter().collect();
    sorted.sort_by_key(|node| names::kebab(&node.name));
    for node in sorted {
        let (kebab, camel) = (names::kebab(&node.name), names::camel(&node.name));
        if node.utils {
            out += &format!(
                "import type {{ Utils as {}Utils }} from \"../nodes/{kebab}.ts\";\n",
                names::pascal(&node.name)
            );
        }
        out += &format!("import defer * as {camel}Node from \"../nodes/{kebab}.ts\";\n");
    }

    out += "\n";
    out += &super::node::state(&state);
    out += "\n";
    let ids: Vec<String> = spec.nodes.iter().map(|node| id(&node.name)).collect();
    out += &union("NodeId", &ids);

    out += "\n/** Per-node utils overrides; omitted nodes use their defaults. */\n";
    out += "export type WalkUtils = {\n";
    for node in spec.nodes.iter().filter(|node| node.utils) {
        out += &format!(
            "  {}?: {}Utils;\n",
            names::camel(&node.name),
            names::pascal(&node.name)
        );
    }
    out += "};\n";

    if unfinished(spec, &state) {
        out += "\n/** A part `amble walk` was not told about: wire it, then delete this. */\n\
                function unwired(part: string): never {\n  \
                  throw new Error(`unwired: ${part}`);\n\
                }\n";
    }

    out += "\nexport function createWalk(utils: WalkUtils = {}) {\n";
    out += "  return ambler<State, NodeId>({\n";
    for node in &spec.nodes {
        out += &entry(spec, node);
    }
    out += "  });\n}\n";

    let start = id(&spec.nodes[0].name);
    let run = format!("  await walk(createWalk(), \"{start}\", ");
    out += "\nif (import.meta.main) {\n";
    out += &initial(&state, 2, &run, ");");
    out += "\n}\n";
    Ok(out)
}

/// The contents of `walks/tests/<name>.test.ts`: a script to fill in and one test to rename.
///
/// # Errors
///
/// Returns a message if the nodes disagree on the state.
pub fn tests(spec: &WalkSpec) -> Result<String, String> {
    let state = spec.state()?;
    let kebab = names::kebab(&spec.name);
    let start = id(&spec.nodes[0].name);
    let scripted = spec.nodes.iter().any(|node| node.utils);
    let mut out = String::from(
        "import { expect, test } from \"bun:test\";\nimport { walk } from \"../../ambler.ts\";\n",
    );
    if scripted {
        out += &format!(
            "import {{ createWalk, type WalkUtils }} from \"../{kebab}.ts\";\n\n\
             /** Plays back scripted inputs and records what the nodes do: give each node the\n \
             * utils it needs, so that nothing real runs in the test. */\n\
             function script(): WalkUtils {{\n  \
               throw new Error(\"not yet implemented\");\n\
             }}\n"
        );
    } else {
        out += &format!("import {{ createWalk }} from \"../{kebab}.ts\";\n");
    }
    let walk = if scripted {
        "createWalk(script())"
    } else {
        "createWalk()"
    };
    let call = format!("  const final = await walk({walk}, \"{start}\", ");
    out += &format!(
        "\n// One test per path through the walk. Rename this after the path it follows.\n\
         test(\"{} walks from start to halt\", async () => {{\n\
         {}\n\
         \n  \
           // TODO: assert the final state and what the script recorded.\n  \
           expect(final).toBeDefined();\n\
         }});\n",
        names::camel(&spec.name),
        object_literal(&state, 2, &call, ");"),
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
    use crate::scaffold::ts::source::node_info;
    use crate::scaffold::walk::parse_edge;

    const COUNTER: &str = include_str!("../../../templates/ts/example/walks/counter.ts");

    fn counter(edges: &[&str]) -> WalkSpec {
        let node = |name: &str, source: &str| node_info(name, source).unwrap();
        WalkSpec::new(
            "counter",
            vec![
                node(
                    "start",
                    include_str!("../../../templates/ts/example/nodes/start.ts"),
                ),
                node(
                    "count",
                    include_str!("../../../templates/ts/example/nodes/count.ts"),
                ),
                node(
                    "stop",
                    include_str!("../../../templates/ts/example/nodes/stop.ts"),
                ),
            ],
            edges.iter().map(|edge| parse_edge(edge).unwrap()).collect(),
        )
    }

    const EDGES: &[&str] = &[
        "start.success=count",
        "start.onError=start",
        "count.count=count",
        "count.stop=stop",
    ];

    #[test]
    fn reproduces_the_reference_counter_walk() {
        let spec = counter(EDGES);

        assert_eq!(validate(&spec), Ok(()));
        assert_eq!(walk(&spec).unwrap(), COUNTER);
    }

    #[test]
    fn leaves_edges_it_was_not_given_unwired() {
        let generated = walk(&counter(&["start.success=count"])).unwrap();

        assert!(generated.contains("function unwired(part: string): never {\n"));
        assert!(generated.contains("onError: unwired(\"START.onError\")"));
        assert!(generated.contains("onStop: unwired(\"COUNT.onStop\")"));
        // A node whose one edge is `onDone` ends the walk unless told otherwise.
        assert!(
            generated.contains("    STOP: () => stopNode.factory({ onDone: null }, utils.stop),\n")
        );
    }

    #[test]
    fn checks_the_edges_and_the_state() {
        let error = |edges: &[&str]| validate(&counter(edges)).unwrap_err();
        assert!(
            error(&["start.nope=count"])
                .contains("`start` has no such outcome (it has onSuccess, onError)")
        );
        assert!(error(&["stop.done=start", "stop.done=count"]).contains("given twice"));
        assert!(error(&["ask.done=start"]).contains("not in `--nodes`"));

        let mut spec = counter(EDGES);
        spec.nodes[2].state[0].ty = "string".to_string();
        assert_eq!(
            validate(&spec).unwrap_err(),
            "the nodes disagree on the state field `count`: `start` has `number`, `stop` has \
             `string`"
        );
    }

    #[test]
    fn writes_a_test_to_finish_and_a_spec() {
        let spec = counter(EDGES);

        let tests = tests(&spec).unwrap();
        assert!(tests.contains("import { createWalk, type WalkUtils } from \"../counter.ts\";\n"));
        assert!(tests.contains("throw new Error(\"not yet implemented\");"));
        assert!(tests.contains(
            "  const final = await walk(createWalk(script()), \"START\", { count: 0 });\n"
        ));

        let md = spec_md(&spec).unwrap();
        assert!(md.contains("### Count\n- TODO: describe what this node does.\n- TODO: when `onCount`, it transitions to `COUNT`.\n"));
        assert!(
            md.contains(
                "### Stop\n- TODO: describe what this node does.\n- Terminates the walk.\n"
            )
        );
        assert!(md.contains("- `count`: TODO: what it holds.\n"));
    }
}
