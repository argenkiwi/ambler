//! Reads what the generators and the linter need out of existing Rust source, using `syn`.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{
    Fields, FnArg, GenericArgument, ImplItem, Item, ItemImpl, ItemTrait, ItemUse, PathArguments,
    ReturnType, TraitItem, TraitItemFn, Type, TypeParamBound, UseTree, WherePredicate,
};

/// Parses a Rust file.
///
/// # Errors
///
/// Returns the line of the first syntax error and what is wrong there.
pub fn parse(src: &str) -> Result<syn::File, (usize, String)> {
    syn::parse_file(src).map_err(|error| (error.span().start().line, error.to_string()))
}

/// The exact source text a span covers.
fn text(src: &str, span: Span) -> &str {
    let offset = |at: proc_macro2::LineColumn| {
        let line_start: usize = src
            .split_inclusive('\n')
            .take(at.line.saturating_sub(1))
            .map(str::len)
            .sum();
        let line = src[line_start..].split('\n').next().unwrap_or_default();
        line_start
            + line
                .char_indices()
                .nth(at.column)
                .map_or(line.len(), |(index, _)| index)
    };
    &src[offset(span.start())..offset(span.end())]
}

/// The source text of a syntax node on one line, with single spaces.
fn snippet(src: &str, node: &impl Spanned) -> String {
    text(src, node.span())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A field a state needs in order to implement a state trait.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

/// A trait method, with the body that implements it on a struct holding the trait's fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method {
    pub signature: String,
    pub body: String,
}

/// A `Has*` state trait from `src/nodes/mod.rs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HasTrait {
    pub name: String,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    /// The paths of the `use` items in that file that the fields' types need, sorted: what a
    /// struct holding the fields must import.
    pub imports: Vec<String>,
}

/// The `Has*` traits declared in `src` (the contents of `src/nodes/mod.rs`).
///
/// # Errors
///
/// Returns a message if `src` is not valid Rust.
pub fn state_traits(src: &str) -> Result<Vec<HasTrait>, String> {
    let file = parse(src).map_err(|(line, error)| format!("line {line}: {error}"))?;
    let imports: Vec<Import> = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Use(item) => Some(use_paths(item)),
            _ => None,
        })
        .flatten()
        .filter(|import| import.name != "*")
        .collect();
    Ok(file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Trait(item) if item.ident.to_string().starts_with("Has") => {
                Some(has_trait(src, item, &imports))
            }
            _ => None,
        })
        .collect())
}

/// The type a state owns for a getter that returns `lent`: `&[T]` is lent by a `Vec<T>`, `&str`
/// by a `String`, `Option<&T>` by an `Option` of the owner of `&T`, `&T` by a `T`.
fn owner(lent: &str) -> String {
    if let Some(inner) = lent
        .strip_prefix("Option<")
        .and_then(|rest| rest.strip_suffix('>'))
        .filter(|inner| inner.starts_with('&'))
    {
        return format!("Option<{}>", owner(inner));
    }
    let Some(referent) = lent.strip_prefix('&') else {
        return lent.to_string();
    };
    let referent = referent.trim_start();
    if referent == "str" {
        "String".to_string()
    } else if let Some(element) = referent
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        format!("Vec<{}>", element.trim())
    } else {
        referent.to_string()
    }
}

/// The body of a getter for `field`, of type `ty`, that returns `returned`.
fn getter(field: &str, ty: &str, returned: &str) -> String {
    match returned.strip_prefix("Option<&") {
        // `Option<&T>` from `Option<T>` lends with `as_ref`; `Option<&str>` from
        // `Option<String>` and the like dereference with `as_deref`.
        Some(lent) if ty.strip_prefix("Option<") == Some(lent) => format!("self.{field}.as_ref()"),
        Some(_) => format!("self.{field}.as_deref()"),
        None if returned.starts_with('&') => format!("&self.{field}"),
        None => format!("self.{field}"),
    }
}

/// The identifiers in a type: `BTreeMap<String, Paving>` has `BTreeMap`, `String`, `Paving`.
fn identifiers(ty: &str) -> impl Iterator<Item = &str> {
    ty.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|word| !word.is_empty())
}

fn has_trait(src: &str, item: &ItemTrait, imports: &[Import]) -> HasTrait {
    let methods: Vec<&TraitItemFn> = item
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(method) => Some(method),
            _ => None,
        })
        .collect();

    // `<field>_mut(&mut self) -> &mut T` and `set_<field>(&mut self, value: T)` name a field.
    let mut fields: Vec<Field> = Vec::new();
    for method in &methods {
        let name = method.sig.ident.to_string();
        let field = if let Some(field) = name.strip_suffix("_mut") {
            match &method.sig.output {
                ReturnType::Type(_, ty) => match &**ty {
                    Type::Reference(reference) => Some((field, snippet(src, &reference.elem))),
                    _ => None,
                },
                ReturnType::Default => None,
            }
        } else if let Some(field) = name.strip_prefix("set_") {
            match method.sig.inputs.iter().nth(1) {
                Some(FnArg::Typed(argument)) => Some((field, snippet(src, &argument.ty))),
                _ => None,
            }
        } else {
            None
        };
        if let Some((field, ty)) = field {
            if fields.iter().all(|known| known.name != field) {
                fields.push(Field {
                    name: field.to_string(),
                    ty,
                });
            }
        }
    }

    let returned = |method: &TraitItemFn| match &method.sig.output {
        ReturnType::Type(_, ty) => Some(snippet(src, &**ty)),
        ReturnType::Default => None,
    };

    // A getter, `<field>(&self) -> T`, with no accessor that names its field: the field owns
    // what the getter lends.
    for method in &methods {
        let name = method.sig.ident.to_string();
        let only_self = method.sig.inputs.len() == 1
            && matches!(method.sig.inputs.first(), Some(FnArg::Receiver(_)));
        let accessor = name.ends_with("_mut") || name.starts_with("set_");
        if only_self && !accessor && fields.iter().all(|known| known.name != name) {
            if let Some(lent) = returned(method) {
                fields.push(Field {
                    ty: owner(&lent),
                    name,
                });
            }
        }
    }

    let field = |name: &str| fields.iter().find(|field| field.name == name);
    let methods = methods
        .iter()
        .map(|method| {
            let name = method.sig.ident.to_string();
            let body = match (name.strip_suffix("_mut"), name.strip_prefix("set_")) {
                (Some(name), _) if field(name).is_some() => format!("&mut self.{name}"),
                (_, Some(name)) if field(name).is_some() => match method.sig.inputs.iter().nth(1) {
                    Some(FnArg::Typed(argument)) => {
                        format!("self.{name} = {};", snippet(src, &argument.pat))
                    }
                    _ => "todo!()".to_string(),
                },
                _ => match (field(&name), returned(method)) {
                    (Some(field), Some(returned)) => getter(&name, &field.ty, &returned),
                    _ => "todo!()".to_string(),
                },
            };
            Method {
                signature: snippet(src, &method.sig),
                body,
            }
        })
        .collect();

    let mut needed: Vec<String> = imports
        .iter()
        .filter(|import| {
            fields
                .iter()
                .any(|field| identifiers(&field.ty).any(|word| word == import.name))
        })
        .map(|import| import.path.clone())
        .collect();
    needed.sort();
    needed.dedup();

    HasTrait {
        name: item.ident.to_string(),
        fields,
        methods,
        imports: needed,
    }
}

/// One variant of a node's outcome enum, and how to match it whatever data it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub name: String,
    /// `""`, `"(..)"` or `" { .. }"`.
    pub pattern: String,
}

/// A node's outcome enum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub ty: String,
    pub variants: Vec<Variant>,
}

/// A name a `use` brings into scope and the path it comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    pub name: String,
    pub path: String,
}

/// What a walk needs to know about a node in order to wire it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeInfo {
    /// The module name, `snake_case`.
    pub name: String,
    /// `None` for a terminal node (`type Outcome = ()`).
    pub outcome: Option<Outcome>,
    /// The error type as written in the node, or `None` if it is `Infallible`.
    pub error: Option<String>,
    /// The signatures of the `Utils` methods to implement, or `None` if the node has no `Utils`.
    pub utils: Option<Vec<String>>,
    /// The `Has*` traits the node requires of the state.
    pub traits: Vec<String>,
    /// Everything the node file imports, to resolve the types in its `Utils` signatures.
    pub imports: Vec<Import>,
}

fn implements(item: &ItemImpl, name: &str) -> bool {
    item.trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .is_some_and(|segment| segment.ident == name)
}

/// Whether `file` has an `impl <name> for …`.
#[must_use]
pub fn has_impl(file: &syn::File, name: &str) -> bool {
    impls(file, name).next().is_some()
}

/// Every `impl <name> for …` in `file`.
pub fn impls<'a>(file: &'a syn::File, name: &'a str) -> impl Iterator<Item = &'a ItemImpl> {
    file.items.iter().filter_map(move |item| match item {
        Item::Impl(item) if implements(item, name) => Some(item),
        _ => None,
    })
}

/// The paths a `use` item imports, one per name.
#[must_use]
pub fn use_paths(item: &ItemUse) -> Vec<Import> {
    fn flatten(tree: &UseTree, prefix: &str, out: &mut Vec<Import>) {
        match tree {
            UseTree::Path(path) => flatten(&path.tree, &format!("{prefix}{}::", path.ident), out),
            UseTree::Name(name) if name.ident == "self" => {
                let path = prefix.trim_end_matches("::");
                out.push(Import {
                    name: path.rsplit("::").next().unwrap_or(path).to_string(),
                    path: path.to_string(),
                });
            }
            UseTree::Name(name) => out.push(Import {
                name: name.ident.to_string(),
                path: format!("{prefix}{}", name.ident),
            }),
            UseTree::Rename(rename) => out.push(Import {
                name: rename.rename.to_string(),
                path: format!("{prefix}{} as {}", rename.ident, rename.rename),
            }),
            UseTree::Group(group) => {
                for tree in &group.items {
                    flatten(tree, prefix, out);
                }
            }
            UseTree::Glob(_) => out.push(Import {
                name: "*".to_string(),
                path: format!("{prefix}*"),
            }),
        }
    }
    let mut out = Vec::new();
    flatten(&item.tree, "", &mut out);
    out
}

/// Reads the node in `src`, the contents of `src/nodes/<name>.rs`.
///
/// # Errors
///
/// Returns a message if the file is not valid Rust or does not look like a node.
pub fn node_info(name: &str, src: &str) -> Result<NodeInfo, String> {
    let file = parse(src).map_err(|(line, error)| format!("line {line}: {error}"))?;
    let node = impls(&file, "Node").next().ok_or("it has no `impl Node`")?;
    let associated = |wanted: &str| {
        node.items
            .iter()
            .find_map(|item| match item {
                ImplItem::Type(item) if item.ident == wanted => Some(&item.ty),
                _ => None,
            })
            .ok_or_else(|| format!("its `impl Node` has no `type {wanted}`"))
    };

    let outcome = match associated("Outcome")? {
        Type::Tuple(tuple) if tuple.elems.is_empty() => None,
        ty => {
            let ty = snippet(src, ty);
            let item = file
                .items
                .iter()
                .find_map(|item| match item {
                    Item::Enum(item) if item.ident == ty => Some(item),
                    _ => None,
                })
                .ok_or_else(|| format!("its outcome type `{ty}` is not an enum in that file"))?;
            let variants = item
                .variants
                .iter()
                .map(|variant| Variant {
                    name: variant.ident.to_string(),
                    pattern: match variant.fields {
                        Fields::Unit => "",
                        Fields::Unnamed(_) => "(..)",
                        Fields::Named(_) => " { .. }",
                    }
                    .to_string(),
                })
                .collect();
            Some(Outcome { ty, variants })
        }
    };

    let error = associated("Error")?;
    let infallible = matches!(error, Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "Infallible"));
    let error = (!infallible).then(|| snippet(src, error));

    // The `S` in `impl<S, U> Node<S> for …`, and what the impl requires of it.
    let state = node
        .trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .and_then(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.first(),
            _ => None,
        })
        .and_then(|argument| match argument {
            GenericArgument::Type(Type::Path(path)) => path.path.get_ident(),
            _ => None,
        });
    let mut bounds: Vec<&TypeParamBound> = Vec::new();
    if let Some(state) = state {
        for parameter in node.generics.type_params() {
            if parameter.ident == *state {
                bounds.extend(&parameter.bounds);
            }
        }
        for predicate in node
            .generics
            .where_clause
            .iter()
            .flat_map(|w| &w.predicates)
        {
            if let WherePredicate::Type(predicate) = predicate {
                let bounded = matches!(&predicate.bounded_ty, Type::Path(path)
                    if path.path.get_ident() == Some(state));
                if bounded {
                    bounds.extend(&predicate.bounds);
                }
            }
        }
    }
    let traits = bounds
        .into_iter()
        .filter_map(|bound| match bound {
            TypeParamBound::Trait(bound) => bound.path.segments.last(),
            _ => None,
        })
        .map(|segment| segment.ident.to_string())
        .filter(|name| !["Send", "Sync", "Sized"].contains(&name.as_str()))
        .collect();

    let utils = file.items.iter().find_map(|item| match item {
        Item::Trait(item) if item.ident == "Utils" => Some(
            item.items
                .iter()
                .filter_map(|item| match item {
                    TraitItem::Fn(method) if method.default.is_none() => {
                        Some(snippet(src, &method.sig))
                    }
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    });

    let imports = file
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Use(item) => Some(use_paths(item)),
            _ => None,
        })
        .flatten()
        .collect();

    Ok(NodeInfo {
        name: name.to_string(),
        outcome,
        error,
        utils,
        traits,
        imports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: &str = include_str!("../../templates/example/start.rs");
    const STOP: &str = include_str!("../../templates/example/stop.rs");

    #[test]
    fn reads_a_fallible_node_with_outcomes() {
        let info = node_info("start", START).unwrap();

        let outcome = info.outcome.unwrap();
        assert_eq!(outcome.ty, "Outcome");
        let variants: Vec<_> = outcome.variants.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(variants, ["Success", "InvalidInput"]);
        assert_eq!(info.error.as_deref(), Some("Error"));
        assert_eq!(info.traits, ["HasCount"]);
        assert_eq!(
            info.utils.unwrap(),
            [
                "fn read_line(&self, message: &str) -> io::Result<Option<String>>",
                "fn print(&self, message: &str)",
            ]
        );
        assert!(info.imports.contains(&Import {
            name: "io".to_string(),
            path: "std::io".to_string(),
        }));
    }

    #[test]
    fn reads_a_terminal_infallible_node() {
        let info = node_info("stop", STOP).unwrap();

        assert_eq!(info.outcome, None);
        assert_eq!(info.error, None);
    }

    #[test]
    fn rejects_a_file_that_is_not_a_node() {
        assert_eq!(
            node_info("nope", "pub fn nope() {}"),
            Err("it has no `impl Node`".to_string())
        );
    }

    #[test]
    fn reads_state_traits_and_how_to_implement_them() {
        let traits = state_traits(
            "pub mod start;\n\npub trait HasCount {\n    fn count(&self) -> i32;\n    \
             fn count_mut(&mut self) -> &mut i32;\n}\n\npub trait HasTopic {\n    \
             fn topic(&self) -> &str;\n    fn set_topic(&mut self, topic: String);\n}\n",
        )
        .unwrap();

        assert_eq!(traits[0].name, "HasCount");
        assert_eq!(traits[0].fields[0].ty, "i32");
        let bodies: Vec<_> = traits[0].methods.iter().map(|m| m.body.as_str()).collect();
        assert_eq!(bodies, ["self.count", "&mut self.count"]);
        assert_eq!(traits[1].fields[0].ty, "String");
        let bodies: Vec<_> = traits[1].methods.iter().map(|m| m.body.as_str()).collect();
        assert_eq!(bodies, ["&self.topic", "self.topic = topic;"]);
    }

    fn bodies(item: &HasTrait) -> Vec<&str> {
        item.methods.iter().map(|m| m.body.as_str()).collect()
    }

    #[test]
    fn an_option_getter_lends_its_contents() {
        let traits = state_traits(
            "pub trait HasMessage {\n    fn message(&self) -> Option<&str>;\n    \
             fn message_mut(&mut self) -> &mut Option<String>;\n}\n\
             pub trait HasLimit {\n    fn limit(&self) -> Option<&u32>;\n    \
             fn limit_mut(&mut self) -> &mut Option<u32>;\n}\n",
        )
        .unwrap();

        assert_eq!(
            bodies(&traits[0]),
            ["self.message.as_deref()", "&mut self.message"]
        );
        assert_eq!(
            bodies(&traits[1]),
            ["self.limit.as_ref()", "&mut self.limit"]
        );
    }

    #[test]
    fn a_getter_alone_names_a_field_owning_what_it_lends() {
        let traits = state_traits(
            "pub trait HasArgs {\n    fn args(&self) -> &[String];\n}\n\
             pub trait HasTitle {\n    fn title(&self) -> &str;\n}\n\
             pub trait HasNote {\n    fn note(&self) -> Option<&str>;\n}\n\
             pub trait HasSize {\n    fn size(&self) -> usize;\n}\n\
             pub trait HasRows {\n    fn rows(&self) -> &Vec<u8>;\n}\n",
        )
        .unwrap();

        let fields: Vec<(&str, &str)> = traits
            .iter()
            .map(|item| (item.fields[0].name.as_str(), item.fields[0].ty.as_str()))
            .collect();
        assert_eq!(
            fields,
            [
                ("args", "Vec<String>"),
                ("title", "String"),
                ("note", "Option<String>"),
                ("size", "usize"),
                ("rows", "Vec<u8>"),
            ]
        );
        let bodies: Vec<&str> = traits.iter().map(|item| bodies(item)[0]).collect();
        assert_eq!(
            bodies,
            [
                "&self.args",
                "&self.title",
                "self.note.as_deref()",
                "self.size",
                "&self.rows"
            ]
        );
    }

    #[test]
    fn a_trait_knows_the_imports_its_fields_need() {
        let traits = state_traits(
            "use crate::scaffold::pave::Paving;\nuse std::collections::{BTreeMap, HashSet};\n\n\
             pub trait HasPaving {\n    fn paving(&self) -> &Paving;\n    \
             fn paving_mut(&mut self) -> &mut Paving;\n}\n\
             pub trait HasSeen {\n    fn seen_mut(&mut self) -> &mut BTreeMap<String, Paving>;\n}\n\
             pub trait HasCount {\n    fn count_mut(&mut self) -> &mut i32;\n}\n",
        )
        .unwrap();

        assert_eq!(traits[0].imports, ["crate::scaffold::pave::Paving"]);
        assert_eq!(
            traits[1].imports,
            [
                "crate::scaffold::pave::Paving",
                "std::collections::BTreeMap"
            ]
        );
        assert!(traits[2].imports.is_empty());
    }
}
