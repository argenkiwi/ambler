//! Generates a TypeScript node: its contract (`State`, `Edge`, `Utils`), a factory that throws
//! `not yet implemented`, and one failing test per edge.
//!
//! The text is written the way Biome formats it, so a fresh node passes `biome check` even
//! before the formatter runs.

use super::source::Field;
use crate::scaffold::names;

/// Biome's line width in generated projects.
const WIDTH: usize = 80;
/// The edge of a node that ends the walk, `amble node --terminal`.
pub const TERMINAL_EDGE: &str = "onDone";

/// What `amble node` was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSpec {
    /// The node's name, `snake_case`.
    pub name: String,
    /// Edge names, `onSuccess`. `[onDone]` for a terminal node.
    pub edges: Vec<String>,
    /// Whether the node ends the walk: its one edge is wired to `null`.
    pub terminal: bool,
    pub state: Vec<Field>,
}

/// The edge an outcome is taken through: `success`, `Success` and `onSuccess` are all
/// `onSuccess`.
#[must_use]
pub fn edge_name(outcome: &str) -> String {
    let pascal = names::pascal(outcome);
    let mut chars = pascal.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some('O'), Some('n'), Some(third)) if third.is_ascii_uppercase() => {
            format!("o{}", &pascal[1..])
        }
        _ => format!("on{pascal}"),
    }
}

/// Whether `name` can be an edge name: `on` and then `PascalCase`.
#[must_use]
pub fn is_edge_name(name: &str) -> bool {
    name.strip_prefix("on").is_some_and(names::is_pascal)
}

/// The id a test wires an edge to: `onInvalidInput` → `INVALID_INPUT`.
#[must_use]
pub fn edge_target(edge: &str) -> String {
    names::shout(&names::snake(edge.strip_prefix("on").unwrap_or(edge)))
}

/// A value of type `ty` to start a test or a walk from, if there is an obvious one.
#[must_use]
pub fn default_value(ty: &str) -> Option<String> {
    let ty = ty.trim();
    let members: Vec<&str> = ty.split('|').map(str::trim).collect();
    if members.contains(&"null") {
        return Some("null".to_string());
    }
    Some(
        match ty {
            "number" => "0",
            "string" => "\"\"",
            "boolean" => "false",
            "bigint" => "0n",
            _ if ty.ends_with("[]") || ty.starts_with("Array<") => "[]",
            _ => return None,
        }
        .to_string(),
    )
}

/// `{ a: 1, b: "" }` on one line if it fits after `indent` and `prefix`, else one property a
/// line.
#[must_use]
pub fn literal(properties: &[String], indent: usize, prefix: &str, suffix: &str) -> String {
    if properties.is_empty() {
        return format!("{prefix}{{}}{suffix}");
    }
    let flat = format!("{prefix}{{ {} }}{suffix}", properties.join(", "));
    if indent + flat.len() <= WIDTH {
        return flat;
    }
    let pad = " ".repeat(indent + 2);
    let mut out = format!("{prefix}{{\n");
    for property in properties {
        out += &format!("{pad}{property},\n");
    }
    out + &" ".repeat(indent) + "}" + suffix
}

/// A value for each required field, as [`literal`] lays it out: the obvious one for its type,
/// or `undefined as never` for a type without one. Optional fields are left out.
#[must_use]
pub fn object_literal(fields: &[Field], indent: usize, prefix: &str, suffix: &str) -> String {
    let properties: Vec<String> = fields
        .iter()
        .filter(|field| !field.optional)
        .map(|field| {
            let value = default_value(&field.ty).unwrap_or_else(|| "undefined as never".into());
            format!("{}: {value}", field.name)
        })
        .collect();
    literal(&properties, indent, prefix, suffix)
}

/// `export type <name> = "a" | "b";`, broken one member a line if it is too long.
#[must_use]
pub fn union(name: &str, members: &[String]) -> String {
    let quoted: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
    let flat = format!("export type {name} = {};\n", quoted.join(" | "));
    if flat.len() <= WIDTH + 1 {
        return flat;
    }
    let mut out = format!("export type {name} =\n");
    for (index, member) in quoted.iter().enumerate() {
        let end = if index + 1 == quoted.len() { ";" } else { "" };
        out += &format!("  | {member}{end}\n");
    }
    out
}

/// The `State` declaration for `fields`.
#[must_use]
pub fn state(fields: &[Field]) -> String {
    if fields.is_empty() {
        return "export type State = object;\n".to_string();
    }
    let mut out = String::from("export interface State {\n");
    for field in fields {
        let optional = if field.optional { "?" } else { "" };
        out += &format!("  {}{optional}: {};\n", field.name, field.ty);
    }
    out + "}\n"
}

/// The contents of `nodes/<name>.ts` for a new node.
#[must_use]
pub fn node(spec: &NodeSpec) -> String {
    format!(
        "import type {{ NodeFactory }} from \"../ambler.ts\";\n\
         \n\
         {state}\
         \n\
         {edge}\
         \n\
         /** The side effects this node needs: one function each, mocked in the tests. */\n\
         export type Utils = Record<string, never>;\n\
         \n\
         const defaultUtils: Utils = {{}};\n\
         \n\
         /** TODO: one sentence on what this node does. */\n\
         export const factory: NodeFactory<State, Edge, Utils> =\n  \
           (_edges, _utils = defaultUtils) =>\n  \
           (_state) => {{\n    \
             throw new Error(\"not yet implemented\");\n  \
           }};\n",
        state = state(&spec.state),
        edge = union("Edge", &spec.edges),
    )
}

/// The edges record a test passes, wired to made-up ids, or `null` for a terminal node.
fn wiring(spec: &NodeSpec) -> Vec<String> {
    spec.edges
        .iter()
        .map(|edge| {
            if spec.terminal {
                format!("{edge}: null")
            } else {
                format!("{edge}: \"{}\"", edge_target(edge))
            }
        })
        .collect()
}

/// `const [next, after] = await factory(<edges>, utils)(state);`, as Biome breaks it.
fn call(spec: &NodeSpec) -> String {
    let edges = wiring(spec);
    let flat = format!(
        "  const [next, after] = await factory({{ {} }}, utils)(state);\n",
        edges.join(", ")
    );
    if flat.len() <= WIDTH + 1 {
        return flat;
    }
    let record = format!("    {{ {} }},\n", edges.join(", "));
    let record = if record.len() <= WIDTH + 1 {
        record
    } else {
        let mut out = String::from("    {\n");
        for edge in &edges {
            out += &format!("      {edge},\n");
        }
        out + "    },\n"
    };
    format!("  const [next, after] = await factory(\n{record}    utils,\n  )(state);\n")
}

/// The contents of `nodes/tests/<name>.test.ts`: one failing test per edge.
#[must_use]
pub fn tests(spec: &NodeSpec) -> String {
    let kebab = names::kebab(&spec.name);
    let mut out = format!(
        "import {{ expect, test }} from \"bun:test\";\n\
         import {{ factory, type State, type Utils }} from \"../{kebab}.ts\";\n\
         \n\
         const utils: Utils = {{}};\n"
    );
    let state = object_literal(&spec.state, 2, "  const state: State = ", ";");
    for edge in &spec.edges {
        let (when, expectation) = if spec.terminal {
            (String::new(), "  expect(next).toBeNull();\n".to_string())
        } else {
            (
                " when TODO: say when".to_string(),
                format!("  expect(next).toEqual(\"{}\");\n", edge_target(edge)),
            )
        };
        out += &format!(
            "\n\
             test(\"{name} takes {edge}{when}\", async () => {{\n\
             {state}\n\
             \n\
             {call}\
             \n\
             {expectation}  \
               expect(after).toEqual(state);\n\
             }});\n",
            name = names::camel(&spec.name),
            call = call(spec),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, ty: &str) -> Field {
        Field {
            name: name.to_string(),
            ty: ty.to_string(),
            optional: false,
        }
    }

    #[test]
    fn names_edges_after_outcomes() {
        assert_eq!(edge_name("success"), "onSuccess");
        assert_eq!(edge_name("invalid_input"), "onInvalidInput");
        assert_eq!(edge_name("onStop"), "onStop");
        assert_eq!(edge_name("once"), "onOnce");
        assert_eq!(edge_target("onInvalidInput"), "INVALID_INPUT");
        assert!(is_edge_name("onStop"));
        assert!(!is_edge_name("stop"));
        assert!(!is_edge_name("on"));
    }

    #[test]
    fn starts_from_an_obvious_value_of_each_type() {
        assert_eq!(default_value("number").as_deref(), Some("0"));
        assert_eq!(default_value("string").as_deref(), Some("\"\""));
        assert_eq!(default_value("Message[]").as_deref(), Some("[]"));
        assert_eq!(default_value("string | null").as_deref(), Some("null"));
        assert_eq!(default_value("Board"), None);
    }

    #[test]
    fn writes_the_contract_and_a_throwing_factory() {
        let spec = NodeSpec {
            name: "count".to_string(),
            edges: vec!["onCount".to_string(), "onStop".to_string()],
            terminal: false,
            state: vec![field("count", "number")],
        };

        let source = node(&spec);

        assert!(source.contains("export interface State {\n  count: number;\n}\n"));
        assert!(source.contains("export type Edge = \"onCount\" | \"onStop\";\n"));
        assert!(source.contains("throw new Error(\"not yet implemented\");"));
    }

    #[test]
    fn breaks_a_long_union_one_member_a_line() {
        let members: Vec<String> = [
            "onSuccessfullyCompletedTheThing",
            "onInvalidInputGiven",
            "onQuit",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(
            union("Edge", &members),
            "export type Edge =\n  | \"onSuccessfullyCompletedTheThing\"\n  | \
             \"onInvalidInputGiven\"\n  | \"onQuit\";\n"
        );
    }

    #[test]
    fn tests_every_edge() {
        let spec = NodeSpec {
            name: "read_move".to_string(),
            edges: vec!["onValid".to_string(), "onInvalid".to_string()],
            terminal: false,
            state: vec![field("board", "string[]")],
        };

        let tests = tests(&spec);

        assert!(tests.contains("from \"../read-move.ts\";\n"));
        assert!(tests.contains("test(\"readMove takes onValid when TODO: say when\""));
        assert!(tests.contains("  const state: State = { board: [] };\n"));
        assert!(tests.contains(
            "  const [next, after] = await factory(\n    { onValid: \"VALID\", onInvalid: \
             \"INVALID\" },\n    utils,\n  )(state);\n"
        ));
        assert!(tests.contains("  expect(next).toEqual(\"INVALID\");\n"));
    }

    #[test]
    fn a_terminal_node_is_wired_to_null() {
        let spec = NodeSpec {
            name: "stop".to_string(),
            edges: vec![TERMINAL_EDGE.to_string()],
            terminal: true,
            state: vec![],
        };

        let tests = tests(&spec);

        assert!(tests.contains("test(\"stop takes onDone\", async () => {\n"));
        assert!(tests.contains("  const state: State = {};\n"));
        assert!(
            tests.contains(
                "  const [next, after] = await factory({ onDone: null }, utils)(state);\n"
            )
        );
        assert!(tests.contains("  expect(next).toBeNull();\n"));
        assert!(node(&spec).contains("export type State = object;\n"));
    }
}
