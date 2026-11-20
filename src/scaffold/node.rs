//! Generates a node skeleton: the types that are its contract, a `todo!()` body and failing tests.

use super::names;
use super::source::{Field, HasTrait};
use std::collections::BTreeSet;

/// Types whose getter returns the value rather than a reference.
const COPY: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
    "u64", "u128", "usize",
];

/// What `amble node` was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSpec {
    /// The module name, `snake_case`.
    pub name: String,
    /// Outcome variants, `PascalCase`. Empty for a terminal node.
    pub outcomes: Vec<String>,
    pub fallible: bool,
    /// The state traits the node requires.
    pub traits: Vec<HasTrait>,
}

/// The name of the state trait for `field`: `count` → `HasCount`.
#[must_use]
pub fn trait_name(field: &str) -> String {
    format!("Has{}", names::pascal(field))
}

/// `ty` with every path in it shortened to its last segment, and the paths to import:
/// `crate::board::Board` is `Board`, imported from `crate::board::Board`.
#[must_use]
pub fn shorten(ty: &str) -> (String, Vec<String>) {
    let mut short = String::new();
    let mut imports = Vec::new();
    let mut rest = ty;
    while !rest.is_empty() {
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':'))
            .unwrap_or(rest.len());
        let (word, after) = rest.split_at(end);
        if word.is_empty() {
            let mut chars = after.chars();
            short.extend(chars.next());
            rest = chars.as_str();
            continue;
        }
        match word.rsplit_once("::") {
            Some((_, last)) if !last.is_empty() => {
                short += last;
                imports.push(word.to_string());
            }
            _ => short += word,
        }
        rest = after;
    }
    (short, imports)
}

/// A new state trait for `field`, with a getter and a `_mut` accessor.
#[must_use]
pub fn has_trait(field: &str, ty: &str) -> String {
    let name = trait_name(field);
    let getter = if COPY.contains(&ty) {
        ty.to_string()
    } else {
        format!("&{ty}")
    };
    format!(
        "/// A state that has a {field} field.\n\
         pub trait {name} {{\n    \
             fn {field}(&self) -> {getter};\n    \
             fn {field}_mut(&mut self) -> &mut {ty};\n\
         }}\n"
    )
}

/// `impl <trait> for <target>`, for a struct that holds the trait's fields.
#[must_use]
pub fn implement(item: &HasTrait, target: &str) -> String {
    let mut out = format!("impl {} for {target} {{\n", item.name);
    for method in &item.methods {
        out += &format!(
            "    {} {{\n        {}\n    }}\n",
            method.signature, method.body
        );
    }
    out + "}\n"
}

/// The fields a struct needs to implement all of `traits`, each once.
#[must_use]
pub fn fields<'a>(traits: impl IntoIterator<Item = &'a HasTrait>) -> Vec<&'a Field> {
    let mut fields: Vec<&Field> = Vec::new();
    for field in traits.into_iter().flat_map(|item| &item.fields) {
        if fields.iter().all(|known| known.name != field.name) {
            fields.push(field);
        }
    }
    fields
}

/// `struct <name> { … }` with the given fields, or a unit struct if there are none.
#[must_use]
pub fn structure(name: &str, fields: &[&Field]) -> String {
    if fields.is_empty() {
        return format!("struct {name};\n");
    }
    let mut out = format!("struct {name} {{\n");
    for field in fields {
        out += &format!("    {}: {},\n", field.name, field.ty);
    }
    out + "}\n"
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.is_empty() {
                "\n".to_string()
            } else {
                format!("    {line}\n")
            }
        })
        .collect()
}

/// The contents of `src/nodes/<name>.rs` for a new node.
#[must_use]
pub fn node(spec: &NodeSpec) -> String {
    let pascal = names::pascal(&spec.name);
    let terminal = spec.outcomes.is_empty();
    let outcome = if terminal { "()" } else { "Outcome" };
    let error = if spec.fallible { "Error" } else { "Infallible" };

    let mut out = String::from("use crate::Node;\n");
    for item in &spec.traits {
        out += &format!("use crate::nodes::{};\n", item.name);
    }
    if !spec.fallible {
        out += "use std::convert::Infallible;\n";
    }

    if !terminal {
        out += &format!(
            "\n/// Where [`{pascal}`] leaves the walk.\n\
             #[derive(Clone, Copy, Debug, PartialEq, Eq)]\n\
             pub enum Outcome {{\n"
        );
        for variant in &spec.outcomes {
            out += &format!("    /// TODO: say when this happens.\n    {variant},\n");
        }
        out += "}\n";
    }

    if spec.fallible {
        out += &format!(
            "\n/// Why [`{pascal}`] could not complete.\n\
             #[derive(Debug, thiserror::Error)]\n\
             pub enum Error {{\n    \
                 #[error(\"TODO: say what could not be done\")]\n    \
                 Io(#[from] std::io::Error),\n\
             }}\n"
        );
    }

    let bounds: String = spec
        .traits
        .iter()
        .map(|item| format!("{} + ", item.name))
        .collect();
    out += &format!(
        "\n/// The side effects [`{pascal}`] needs: one method each, mocked in the tests.\n\
         pub trait Utils {{}}\n\
         \n\
         #[derive(Clone, Copy, Debug, Default)]\n\
         pub struct SystemUtils;\n\
         \n\
         impl Utils for SystemUtils {{}}\n\
         \n\
         /// TODO: one sentence on what this node does.\n\
         #[derive(Clone, Debug, Default)]\n\
         pub struct {pascal}<U = SystemUtils> {{\n    \
             utils: U,\n\
         }}\n\
         \n\
         impl {pascal} {{\n    \
             #[must_use]\n    \
             pub fn new() -> Self {{\n        \
                 Self::default()\n    \
             }}\n\
         }}\n\
         \n\
         impl<U> {pascal}<U> {{\n    \
             #[must_use]\n    \
             pub fn with_utils(utils: U) -> Self {{\n        \
                 Self {{ utils }}\n    \
             }}\n\
         }}\n\
         \n\
         impl<S, U> Node<S> for {pascal}<U>\n\
         where\n    \
             S: {bounds}Send,\n    \
             U: Utils + Sync,\n\
         {{\n    \
             type Outcome = {outcome};\n    \
             type Error = {error};\n\
         \n    \
             async fn run(&self, _state: &mut S) -> Result<{outcome}, {error}> {{\n        \
                 todo!()\n    \
             }}\n\
         }}\n"
    );

    out + "\n" + &tests(spec)
}

fn tests(spec: &NodeSpec) -> String {
    let pascal = names::pascal(&spec.name);
    let fields = fields(&spec.traits);

    let mut body = String::from("use super::*;\n");
    let imports: BTreeSet<&String> = spec.traits.iter().flat_map(|item| &item.imports).collect();
    for import in imports {
        body += &format!("use {import};\n");
    }
    body += "\n";
    body += &structure("MockState", &fields);
    for item in &spec.traits {
        body += "\n";
        body += &implement(item, "MockState");
    }
    body += "\nstruct MockUtils;\n\nimpl Utils for MockUtils {}\n";

    let state = if fields.is_empty() {
        "MockState".to_string()
    } else {
        let values: Vec<String> = fields
            .iter()
            .map(|field| format!("{}: Default::default()", field.name))
            .collect();
        format!("MockState {{ {} }}", values.join(", "))
    };
    let run = format!("{pascal}::with_utils(MockUtils).run(&mut state).await");

    body += "\n// One test per branch. Rename each after the behaviour it proves, set up the state and\n\
             // the mocks that lead to it, and assert the outcome and the state.\n";
    let mut test = |name: &str, act: String, assert: String| {
        body += &format!(
            "#[tokio::test]\nasync fn {name}() {{\n    let mut state = {state};\n\n    \
             {act}\n{assert}}}\n\n"
        );
    };
    if spec.outcomes.is_empty() {
        let act = if spec.fallible {
            format!("{run}.unwrap();")
        } else {
            format!("let Ok(()) = {run};")
        };
        test("completes", act, String::new());
    }
    for variant in &spec.outcomes {
        let act = if spec.fallible {
            format!("let outcome = {run}.unwrap();")
        } else {
            format!("let Ok(outcome) = {run};")
        };
        test(
            &format!("returns_{}", names::snake(variant)),
            act,
            format!("\n    assert_eq!(outcome, Outcome::{variant});\n"),
        );
    }
    if spec.fallible {
        test(
            "fails_with_io",
            format!("let result = {run};"),
            "\n    assert!(matches!(result, Err(Error::Io(_))));\n".to_string(),
        );
    }

    format!(
        "#[cfg(test)]\nmod tests {{\n{}}}\n",
        indent(body.trim_end())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::source::state_traits;

    fn count() -> Vec<HasTrait> {
        state_traits(&has_trait("count", "i32")).unwrap()
    }

    #[test]
    fn copy_types_are_returned_by_value_and_others_by_reference() {
        assert!(has_trait("count", "i32").contains("fn count(&self) -> i32;"));
        let name = has_trait("user_name", "String");
        assert!(name.contains("pub trait HasUserName {"));
        assert!(name.contains("fn user_name(&self) -> &String;"));
        assert!(name.contains("fn user_name_mut(&mut self) -> &mut String;"));
    }

    #[test]
    fn a_node_has_its_contract_an_unimplemented_body_and_a_test_per_branch() {
        let source = node(&NodeSpec {
            name: "ask_number".to_string(),
            outcomes: vec!["Success".to_string(), "InvalidInput".to_string()],
            fallible: true,
            traits: count(),
        });

        syn::parse_file(&source).unwrap();
        assert!(source.starts_with("use crate::Node;\nuse crate::nodes::HasCount;\n\n"));
        assert!(source.contains("pub enum Outcome {"));
        assert!(source.contains("    InvalidInput,\n"));
        assert!(source.contains("pub enum Error {"));
        assert!(source.contains("pub struct AskNumber<U = SystemUtils> {"));
        assert!(source.contains("    S: HasCount + Send,\n"));
        assert!(source.contains("    type Error = Error;\n"));
        assert!(source.contains("-> Result<Outcome, Error> {\n        todo!()\n"));
        assert!(source.contains("    impl HasCount for MockState {\n"));
        assert!(source.contains("async fn returns_success() {"));
        assert!(source.contains("async fn returns_invalid_input() {"));
        assert!(source.contains("async fn fails_with_io() {"));
    }

    #[test]
    fn a_terminal_infallible_node_has_no_outcome_enum() {
        let source = node(&NodeSpec {
            name: "stop".to_string(),
            outcomes: Vec::new(),
            fallible: false,
            traits: Vec::new(),
        });

        syn::parse_file(&source).unwrap();
        assert!(!source.contains("enum Outcome"));
        assert!(source.contains("use std::convert::Infallible;\n"));
        assert!(source.contains("    S: Send,\n"));
        assert!(source.contains("    type Outcome = ();\n"));
        assert!(source.contains("    struct MockState;\n"));
        assert!(source.contains("let Ok(()) = Stop::with_utils(MockUtils).run(&mut state).await;"));
    }

    #[test]
    fn a_path_in_a_type_becomes_an_import() {
        assert_eq!(shorten("i32"), ("i32".to_string(), Vec::new()));
        assert_eq!(
            shorten("crate::scaffold::pave::Paving"),
            (
                "Paving".to_string(),
                vec!["crate::scaffold::pave::Paving".to_string()]
            )
        );
        assert_eq!(
            shorten("std::collections::HashMap<String, ::app::Cell>"),
            (
                "HashMap<String, Cell>".to_string(),
                vec![
                    "std::collections::HashMap".to_string(),
                    "::app::Cell".to_string()
                ]
            )
        );
    }

    #[test]
    fn the_mock_state_imports_what_its_fields_need() {
        let traits = state_traits(
            "use crate::board::Board;\n\npub trait HasBoard {\n    fn board(&self) -> &Board;\n    \
             fn board_mut(&mut self) -> &mut Board;\n}\n",
        )
        .unwrap();

        let source = node(&NodeSpec {
            name: "place".to_string(),
            outcomes: Vec::new(),
            fallible: false,
            traits,
        });

        syn::parse_file(&source).unwrap();
        assert!(source.contains("    use super::*;\n    use crate::board::Board;\n\n"));
        assert!(source.contains("        board: Board,\n"));
    }
}
