//! Generates a walk from the nodes it wires: state, ids, the exhaustive `step`, its spec and
//! its runner.

use super::names;
use super::node::{fields, implement, structure};
use super::source::{HasTrait, NodeInfo};
use std::collections::BTreeSet;

/// `--edge <from>.<Outcome>=<to>`: where one outcome of a node leads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: String,
    pub outcome: String,
    /// The next node, or `None` to end the walk (`=halt`).
    pub to: Option<String>,
}

/// What `amble walk` was asked for, with what it read from the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkSpec {
    /// The module name, `snake_case`.
    pub name: String,
    /// The nodes to wire; the first is where the walk starts.
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<Edge>,
    /// Every state trait in `src/nodes/mod.rs`.
    pub traits: Vec<HasTrait>,
}

/// Parses `start.Success=count` or `count.Finished=halt`.
///
/// # Errors
///
/// Returns a message showing the expected shape.
pub fn parse_edge(text: &str) -> Result<Edge, String> {
    let bad =
        || format!("`--edge {text}` should look like `--edge start.Success=count` (or `=halt`)");
    let (source, to) = text.split_once('=').ok_or_else(&bad)?;
    let (from, outcome) = source.split_once('.').ok_or_else(&bad)?;
    if from.is_empty() || outcome.is_empty() || to.is_empty() {
        return Err(bad());
    }
    Ok(Edge {
        from: from.to_string(),
        outcome: names::pascal(outcome),
        to: (to != "halt").then(|| to.to_string()),
    })
}

/// Checks that the nodes and edges make sense together.
///
/// # Errors
///
/// Returns a message naming the first problem.
pub fn validate(spec: &WalkSpec) -> Result<(), String> {
    let node = |name: &str| spec.nodes.iter().find(|node| node.name == name);
    for (index, info) in spec.nodes.iter().enumerate() {
        if spec.nodes[..index]
            .iter()
            .any(|other| other.name == info.name)
        {
            return Err(format!("node `{}` is listed twice", info.name));
        }
        if info.name == spec.name {
            return Err(format!(
                "the walk and the node `{}` cannot share a name",
                info.name
            ));
        }
    }
    for (index, edge) in spec.edges.iter().enumerate() {
        let from = node(&edge.from)
            .ok_or_else(|| format!("edge from `{}`: it is not in `--nodes`", edge.from))?;
        let Some(outcome) = &from.outcome else {
            return Err(format!(
                "edge from `{}`: it is a terminal node, it has no outcomes",
                edge.from
            ));
        };
        if outcome
            .variants
            .iter()
            .all(|variant| variant.name != edge.outcome)
        {
            let known: Vec<&str> = outcome.variants.iter().map(|v| v.name.as_str()).collect();
            return Err(format!(
                "edge `{}.{}`: `{}` has no such outcome (it has {})",
                edge.from,
                edge.outcome,
                edge.from,
                known.join(", ")
            ));
        }
        if let Some(to) = &edge.to {
            if node(to).is_none() {
                return Err(format!(
                    "edge `{}.{}={to}`: `{to}` is not in `--nodes`",
                    edge.from, edge.outcome
                ));
            }
        }
        let repeated = spec.edges[..index]
            .iter()
            .any(|other| other.from == edge.from && other.outcome == edge.outcome);
        if repeated {
            return Err(format!(
                "edge `{}.{}` is given twice",
                edge.from, edge.outcome
            ));
        }
    }
    Ok(())
}

/// The generic parameter standing for the utils of the `index`th node that has any.
fn letter(index: usize) -> String {
    u8::try_from(index)
        .ok()
        .filter(|index| *index < 26)
        .map_or_else(
            || format!("U{index}"),
            |index| char::from(b'A' + index).to_string(),
        )
}

/// How the walk refers to a node's error type.
fn error_path(info: &NodeInfo, error: &str) -> String {
    if error.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        format!("{}::{error}", info.name)
    } else {
        error.to_string()
    }
}

struct Wiring<'a> {
    spec: &'a WalkSpec,
    pascal: String,
    /// The state traits the nodes require, sorted, with their definition when it was found.
    traits: Vec<(String, Option<&'a HasTrait>)>,
    /// One generic parameter per node that has utils, parallel to `spec.nodes`.
    generics: Vec<Option<String>>,
    /// The nodes that can fail, with the path of their error type.
    fallible: Vec<(&'a NodeInfo, String)>,
}

impl<'a> Wiring<'a> {
    fn new(spec: &'a WalkSpec) -> Self {
        let names: BTreeSet<&String> = spec.nodes.iter().flat_map(|node| &node.traits).collect();
        let traits = names
            .into_iter()
            .map(|name| {
                let found = spec.traits.iter().find(|item| item.name == *name);
                (name.clone(), found)
            })
            .collect();
        let mut used = 0;
        let generics = spec
            .nodes
            .iter()
            .map(|node| {
                node.utils.as_ref().map(|_| {
                    used += 1;
                    letter(used - 1)
                })
            })
            .collect();
        let fallible = spec
            .nodes
            .iter()
            .filter_map(|node| {
                let error = node.error.as_ref()?;
                Some((node, error_path(node, error)))
            })
            .collect();
        Self {
            spec,
            pascal: names::pascal(&spec.name),
            traits,
            generics,
            fallible,
        }
    }

    fn parameters(&self) -> Vec<&str> {
        self.generics.iter().flatten().map(String::as_str).collect()
    }

    /// `<A, B>`, or nothing when no node has utils.
    fn arguments(&self, of: impl Fn(&str) -> String) -> String {
        let parameters = self.parameters();
        if parameters.is_empty() {
            String::new()
        } else {
            let list: Vec<String> = parameters.into_iter().map(of).collect();
            format!("<{}>", list.join(", "))
        }
    }

    fn target(&self, node: &NodeInfo, variant: &str) -> String {
        let edge = self
            .spec
            .edges
            .iter()
            .find(|edge| edge.from == node.name && edge.outcome == variant);
        match edge {
            Some(Edge { to: Some(to), .. }) => {
                format!("Transition::Next(Id::{})", names::pascal(to))
            }
            Some(Edge { to: None, .. }) => "Transition::Halt".to_string(),
            None => format!("todo!(\"wire {}::Outcome::{variant}\")", node.name),
        }
    }

    fn header(&self) -> String {
        let name = &self.spec.name;
        let mut out = format!(
            "//! TODO: one sentence on what this walk does.\n//!\n//! See `specs/{name}.md`.\n\n"
        );
        for (item, _) in &self.traits {
            out += &format!("use crate::nodes::{item};\n");
        }
        let mut nodes: Vec<&NodeInfo> = self.spec.nodes.iter().collect();
        nodes.sort_by(|a, b| a.name.cmp(&b.name));
        for node in nodes {
            out += &format!(
                "use crate::nodes::{}::{{self, {}}};\n",
                node.name,
                names::pascal(&node.name)
            );
        }
        out += "use crate::{Node, Transition, Walk};\n";
        if self.fallible.is_empty() {
            out += "use std::convert::Infallible;\n";
        }
        // What the state's fields need; rustfmt puts them in order.
        let imports: BTreeSet<&String> = self
            .traits
            .iter()
            .filter_map(|(_, item)| *item)
            .flat_map(|item| &item.imports)
            .collect();
        for import in imports {
            out += &format!("use {import};\n");
        }
        out
    }

    fn error(&self) -> String {
        match self.fallible.as_slice() {
            [] => "/// No node in this walk can fail.\npub type Error = Infallible;\n".to_string(),
            [(node, path)] => format!(
                "/// What can stop the walk early. Only the `{}` node can fail.\n\
                 pub type Error = {path};\n",
                node.name
            ),
            many => {
                let mut out = String::from(
                    "/// What can stop the walk early.\n\
                     #[derive(Debug, thiserror::Error)]\n\
                     pub enum Error {\n",
                );
                for (node, path) in many {
                    out += &format!(
                        "    #[error(transparent)]\n    {}(#[from] {path}),\n",
                        names::pascal(&node.name)
                    );
                }
                out + "}\n"
            }
        }
    }

    fn state(&self) -> String {
        let found = self.traits.iter().filter_map(|(_, item)| *item);
        let mut out = String::from("#[derive(Clone, Debug, Default)]\n");
        out += &structure("State", &fields(found));
        for (name, item) in &self.traits {
            out += "\n";
            out += &match item {
                Some(item) => implement(item, "State"),
                // Not declared in src/nodes/mod.rs: leave an impl the compiler will ask to fill.
                None => format!("impl {name} for State {{}}\n"),
            };
        }
        out
    }

    fn ids(&self) -> String {
        let mut out = String::from("#[derive(Clone, Copy, Debug, PartialEq, Eq)]\nenum Id {\n");
        for node in &self.spec.nodes {
            out += &format!("    {},\n", names::pascal(&node.name));
        }
        out + "}\n"
    }

    fn structure(&self) -> String {
        let pascal = &self.pascal;
        let defaults: Vec<String> = self
            .spec
            .nodes
            .iter()
            .zip(&self.generics)
            .filter_map(|(node, generic)| {
                generic
                    .as_ref()
                    .map(|generic| format!("{generic} = {}::SystemUtils", node.name))
            })
            .collect();
        let generics = if defaults.is_empty() {
            String::new()
        } else {
            format!("<{}>", defaults.join(", "))
        };
        let mut out = format!("#[derive(Default)]\nstruct {pascal}{generics} {{\n");
        for (node, generic) in self.spec.nodes.iter().zip(&self.generics) {
            let ty = names::pascal(&node.name);
            match generic {
                Some(generic) => out += &format!("    {}: {ty}<{generic}>,\n", node.name),
                None => out += &format!("    {}: {ty},\n", node.name),
            }
        }
        out += &format!(
            "}}\n\nimpl {pascal} {{\n    \
             /// Production wiring: every node uses its `SystemUtils`.\n    \
             fn new() -> Self {{\n        Self::default()\n    }}\n}}\n"
        );
        out
    }

    fn walk(&self) -> String {
        let pascal = &self.pascal;
        let parameters = self.arguments(ToString::to_string);
        let mut out = format!("impl{parameters} Walk for {pascal}{parameters}\n");
        if !parameters.is_empty() {
            out += "where\n";
            for (node, generic) in self.spec.nodes.iter().zip(&self.generics) {
                if let Some(generic) = generic {
                    out += &format!("    {generic}: {}::Utils + Sync,\n", node.name);
                }
            }
        }
        out += "{\n    type Id = Id;\n    type State = State;\n    type Error = Error;\n\n    \
                async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {\n        \
                Ok(match id {\n";
        for node in &self.spec.nodes {
            out += &self.arm(node);
        }
        out + "        })\n    }\n}\n"
    }

    fn arm(&self, node: &NodeInfo) -> String {
        let id = names::pascal(&node.name);
        let run = format!("self.{}.run(state).await", node.name);
        let Some(outcome) = &node.outcome else {
            let act = if node.error.is_some() {
                format!("{run}?;")
            } else {
                format!("let Ok(()) = {run};")
            };
            return format!("Id::{id} => {{\n{act}\nTransition::Halt\n}}\n");
        };
        let arms: String = outcome
            .variants
            .iter()
            .map(|variant| {
                format!(
                    "{}::{}::{}{} => {},\n",
                    node.name,
                    outcome.ty,
                    variant.name,
                    variant.pattern,
                    self.target(node, &variant.name)
                )
            })
            .collect();
        if node.error.is_some() {
            format!("Id::{id} => match {run}? {{\n{arms}}},\n")
        } else {
            format!("Id::{id} => {{\nlet Ok(outcome) = {run};\nmatch outcome {{\n{arms}}}\n}}\n")
        }
    }

    fn run(&self) -> String {
        let (name, pascal) = (&self.spec.name, &self.pascal);
        let start = names::pascal(&self.spec.nodes[0].name);
        let errors = if self.fallible.is_empty() {
            "Never: no node in this walk can fail.".to_string()
        } else {
            let nodes: Vec<String> = self
                .fallible
                .iter()
                .map(|(node, _)| format!("`{}`", node.name))
                .collect();
            format!("Returns [`Error`] if {} fails.", nodes.join(" or "))
        };
        format!(
            "/// Runs the {name} walk.\n///\n/// # Errors\n///\n/// {errors}\n\
             pub async fn run() -> Result<(), Error> {{\n    \
             {pascal}::new().run(Id::{start}, &mut State::default()).await\n}}\n"
        )
    }

    fn tests(&self) -> String {
        let (name, pascal) = (&self.spec.name, &self.pascal);
        let start = names::pascal(&self.spec.nodes[0].name);
        let scripted = !self.parameters().is_empty();

        // The types the nodes' `Utils` signatures mention, imported the way each node does.
        let mut imports: BTreeSet<String> = BTreeSet::new();
        for node in &self.spec.nodes {
            for signature in node.utils.iter().flatten() {
                let mentioned: BTreeSet<&str> = signature
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .collect();
                for import in &node.imports {
                    if mentioned.contains(import.name.as_str()) {
                        imports.insert(format!("use {};\n", import.path));
                    }
                }
            }
        }

        let mut out = String::from("#[cfg(test)]\nmod tests {\nuse super::*;\n");
        out.extend(imports);
        if scripted {
            out += "\n/// Plays back scripted inputs and records what the nodes do. Give it a field\n\
                    /// per input to play back (a `Mutex<VecDeque<_>>`) and per effect to record.\n\
                    struct Script;\n";
            for node in &self.spec.nodes {
                let Some(signatures) = &node.utils else {
                    continue;
                };
                out += &format!("\nimpl {}::Utils for &Script {{\n", node.name);
                for signature in signatures {
                    let body = if signature.contains("-> impl Future") {
                        "async { todo!() }"
                    } else {
                        "todo!()"
                    };
                    out += &format!("{signature} {{\n{body}\n}}\n");
                }
                out += "}\n";
            }
            let scripts = self.arguments(|_| "&Script".to_string());
            out += &format!("\nfn {name}(script: &Script) -> {pascal}{scripts} {{\n{pascal} {{\n");
            for (node, generic) in self.spec.nodes.iter().zip(&self.generics) {
                let ty = names::pascal(&node.name);
                match generic {
                    Some(_) => out += &format!("{}: {ty}::with_utils(script),\n", node.name),
                    None => out += &format!("{}: {ty}::default(),\n", node.name),
                }
            }
            out += "}\n}\n";
        }

        let walk = if scripted {
            format!("{name}(&script)")
        } else {
            format!("{pascal}::new()")
        };
        let run = format!("{walk}.run(Id::{start}, &mut state).await");
        let act = if self.fallible.is_empty() {
            format!("let Ok(()) = {run};")
        } else {
            format!("{run}.unwrap();")
        };
        out += "\n// One test per path through the walk. Rename this after the path it follows.\n\
                #[tokio::test]\nasync fn walks_from_start_to_halt() {\n";
        if scripted {
            out += "let script = Script;\n";
        }
        out += &format!(
            "let mut state = State::default();\n\n{act}\n\n\
             // TODO: assert the final state and what the script recorded.\n}}\n}}\n"
        );
        out
    }
}

/// The contents of `src/walks/<name>.rs`. Run it through rustfmt: it is not indented.
#[must_use]
pub fn walk(spec: &WalkSpec) -> String {
    let wiring = Wiring::new(spec);
    [
        wiring.header(),
        wiring.error(),
        wiring.state(),
        wiring.ids(),
        wiring.structure(),
        wiring.walk(),
        wiring.run(),
        wiring.tests(),
    ]
    .join("\n")
}

/// The contents of `specs/<name>.md`: one section per node, with the edges already known.
#[must_use]
pub fn spec_md(spec: &WalkSpec) -> String {
    let wiring = Wiring::new(spec);
    let mut out = String::from(
        "# Program Specifications\n\nTODO: describe the program and its purpose.\n\n## Nodes\n",
    );
    for (index, node) in spec.nodes.iter().enumerate() {
        out += &format!("\n### {}\n", names::title(&node.name));
        if index == 0 {
            out += "- Initial node.\n";
        }
        out += "- TODO: describe what this node does.\n";
        match &node.outcome {
            None => out += "- Terminates the walk.\n",
            Some(outcome) => {
                for variant in &outcome.variants {
                    let edge = spec
                        .edges
                        .iter()
                        .find(|edge| edge.from == node.name && edge.outcome == variant.name);
                    out += &match edge {
                        Some(Edge { to: Some(to), .. }) => format!(
                            "- TODO: when `{}`, it transitions to `{}`.\n",
                            variant.name,
                            names::shout(to)
                        ),
                        Some(Edge { to: None, .. }) => {
                            format!("- TODO: when `{}`, it terminates.\n", variant.name)
                        }
                        None => format!(
                            "- TODO: when `{}`, it transitions to (not wired yet).\n",
                            variant.name
                        ),
                    };
                }
            }
        }
    }
    out += "\n## Shared State\n\n";
    let fields = fields(wiring.traits.iter().filter_map(|(_, item)| *item));
    if fields.is_empty() {
        out += "TODO: describe the state the nodes share, if any.\n";
    } else {
        out += "The shared state consists of:\n";
        for field in fields {
            out += &format!("- `{}`: TODO: what it holds.\n", field.name);
        }
    }
    out
}

/// The binary that runs a walk: `src/main.rs` or `src/bin/<walk>.rs`.
#[must_use]
pub fn runner(crate_name: &str, walk: &str) -> String {
    format!(
        "//! Runs the `{walk}` walk; see `specs/{walk}.md`.\n\n\
         #[tokio::main]\n\
         async fn main() -> Result<(), {crate_name}::walks::{walk}::Error> {{\n    \
             {crate_name}::walks::{walk}::run().await\n\
         }}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::source::{node_info, state_traits};

    fn counter(edges: &[&str]) -> WalkSpec {
        let node = |name: &str, source: &str| node_info(name, source).unwrap();
        WalkSpec {
            name: "counter".to_string(),
            nodes: vec![
                node("start", include_str!("../../templates/example/start.rs")),
                node("count", include_str!("../../templates/example/count.rs")),
                node("stop", include_str!("../../templates/example/stop.rs")),
            ],
            edges: edges.iter().map(|edge| parse_edge(edge).unwrap()).collect(),
            traits: state_traits(
                "pub trait HasCount {\n    fn count(&self) -> i32;\n    \
                 fn count_mut(&mut self) -> &mut i32;\n}\n",
            )
            .unwrap(),
        }
    }

    const EDGES: &[&str] = &[
        "start.Success=count",
        "start.invalid_input=start",
        "count.Again=count",
        "count.Finished=stop",
    ];

    /// Drops indentation and blank lines, which rustfmt decides, to compare the code alone.
    fn code(source: &str) -> Vec<&str> {
        source
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect()
    }

    #[test]
    fn parses_edges() {
        assert_eq!(
            parse_edge("start.invalid_input=start").unwrap(),
            Edge {
                from: "start".to_string(),
                outcome: "InvalidInput".to_string(),
                to: Some("start".to_string()),
            }
        );
        assert_eq!(parse_edge("count.Finished=halt").unwrap().to, None);
        assert!(parse_edge("start=count").is_err());
        assert!(parse_edge("start.Success").is_err());
    }

    #[test]
    fn reproduces_the_reference_counter_walk() {
        let reference = include_str!("../../templates/example/counter.rs");
        let reference = reference.split("#[cfg(test)]").next().unwrap();

        let generated = walk(&counter(EDGES));
        let generated = generated.split("#[cfg(test)]").next().unwrap();

        syn::parse_file(generated).unwrap();
        assert_eq!(code(generated), code(reference));
    }

    #[test]
    fn edges_not_given_are_left_to_wire() {
        let generated = walk(&counter(&["start.Success=count"]));

        syn::parse_file(&generated).unwrap();
        assert!(generated.contains("start::Outcome::Success => Transition::Next(Id::Count),"));
        assert!(generated.contains(
            "start::Outcome::InvalidInput => todo!(\"wire start::Outcome::InvalidInput\"),"
        ));
    }

    #[test]
    fn the_wiring_test_scripts_every_node_and_imports_what_the_signatures_mention() {
        let generated = walk(&counter(EDGES));

        assert!(generated.contains("use std::io;\n"));
        assert!(generated.contains("use std::time::Duration;\n"));
        assert!(generated.contains("impl start::Utils for &Script {\n"));
        assert!(generated.contains(
            "fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {\n\
             async { todo!() }\n"
        ));
        assert!(
            generated
                .contains("fn counter(script: &Script) -> Counter<&Script, &Script, &Script> {")
        );
    }

    #[test]
    fn the_state_imports_what_its_fields_need() {
        let mut spec = counter(EDGES);
        spec.traits = state_traits(
            "use crate::tally::Tally;\n\npub trait HasCount {\n    fn count(&self) -> &Tally;\n    \
             fn count_mut(&mut self) -> &mut Tally;\n}\n",
        )
        .unwrap();

        let generated = walk(&spec);
        let body = generated.split("#[cfg(test)]").next().unwrap();

        syn::parse_file(&generated).unwrap();
        assert!(body.contains("use crate::tally::Tally;\n"));
        assert!(body.contains("    count: Tally,\n"));
    }

    #[test]
    fn several_fallible_nodes_get_an_error_enum() {
        let mut spec = counter(EDGES);
        spec.nodes[2].error = Some("Error".to_string());

        let generated = walk(&spec);

        syn::parse_file(&generated).unwrap();
        assert!(generated.contains(
            "pub enum Error {\n    #[error(transparent)]\n    Start(#[from] start::Error),"
        ));
        assert!(generated.contains("Stop(#[from] stop::Error),"));
        assert!(generated.contains("self.stop.run(state).await?;\nTransition::Halt"));
    }

    #[test]
    fn rejects_edges_that_do_not_fit_the_nodes() {
        let error = |edge: &str| validate(&counter(&[edge])).unwrap_err();

        assert!(error("nope.Success=count").contains("not in `--nodes`"));
        assert!(error("stop.Success=count").contains("terminal"));
        assert!(error("start.Nope=count").contains("it has Success, InvalidInput"));
        assert!(error("start.Success=nope").contains("`nope` is not in `--nodes`"));
        assert_eq!(validate(&counter(EDGES)), Ok(()));
    }

    #[test]
    fn the_spec_has_a_section_per_node_and_the_shared_fields() {
        let spec = spec_md(&counter(EDGES));

        assert!(spec.starts_with("# Program Specifications\n"));
        assert!(spec.contains("\n### Start\n- Initial node.\n"));
        assert!(spec.contains("- TODO: when `Finished`, it transitions to `STOP`.\n"));
        assert!(spec.contains(
            "\n### Stop\n- TODO: describe what this node does.\n- Terminates the walk.\n"
        ));
        assert!(spec.contains("\n## Shared State\n\nThe shared state consists of:\n- `count`: "));
    }

    #[test]
    fn the_runner_names_the_crate_it_is_in() {
        assert!(runner("my_app", "counter").contains(
            "async fn main() -> Result<(), my_app::walks::counter::Error> {\n    \
             my_app::walks::counter::run().await\n}"
        ));
    }
}
