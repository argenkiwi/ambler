//! Reads TypeScript sources with oxc: what a module declares and exports, a node's contract
//! (`State`, `Edge`, `Utils`, `factory`) and a walk's node ids.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPatternKind, Declaration, ImportDeclarationSpecifier, ImportOrExportKind, ImportPhase,
    PropertyKey, Statement, TSLiteral, TSSignature, TSType,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};

/// What kind of thing a top-level item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Interface,
    TypeAlias,
    Const,
    Variable,
    Function,
    Class,
    Other,
}

/// A top-level declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub kind: Kind,
    pub exported: bool,
    /// 1-based.
    pub line: usize,
}

/// An `import` statement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    /// The module it imports from, as written: `../ambler.ts`.
    pub source: String,
    /// `import type …`.
    pub type_only: bool,
    /// `import defer …`.
    pub deferred: bool,
    /// The local names it binds.
    pub names: Vec<String>,
    pub line: usize,
}

/// A property of a `State` interface or object type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// As written, `camelCase`.
    pub name: String,
    /// The type as written, `number` or `Message[]`.
    pub ty: String,
    /// `name?: T`.
    pub optional: bool,
}

/// What a module declares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Module {
    pub items: Vec<Item>,
    pub imports: Vec<Import>,
    /// The string literals of each type alias that is a union of them (or one of them), by name:
    /// `type Edge = "onA" | "onB"` gives `("Edge", ["onA", "onB"])`.
    pub unions: Vec<(String, Vec<String>)>,
    /// The properties of each interface or object type alias, by name.
    pub objects: Vec<(String, Vec<Field>)>,
}

impl Module {
    #[must_use]
    pub fn item(&self, name: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.name == name)
    }

    #[must_use]
    pub fn exported(&self, name: &str) -> Option<&Item> {
        self.item(name).filter(|item| item.exported)
    }

    #[must_use]
    pub fn union(&self, name: &str) -> Option<&[String]> {
        self.unions
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, members)| members.as_slice())
    }

    #[must_use]
    pub fn object(&self, name: &str) -> Option<&[Field]> {
        self.objects
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, fields)| fields.as_slice())
    }
}

/// The 1-based line `offset` is on.
#[must_use]
pub fn line_of(src: &str, offset: u32) -> usize {
    let offset = usize::try_from(offset).unwrap_or(usize::MAX).min(src.len());
    src[..offset].matches('\n').count() + 1
}

fn text(src: &str, span: Span) -> String {
    src.get(span.start as usize..span.end as usize)
        .unwrap_or_default()
        .to_string()
}

fn key_name(key: &PropertyKey) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn fields(src: &str, members: &[TSSignature]) -> Vec<Field> {
    members
        .iter()
        .filter_map(|member| match member {
            TSSignature::TSPropertySignature(property) => Some(Field {
                name: key_name(&property.key)?,
                ty: property.type_annotation.as_ref().map_or_else(
                    || "unknown".to_string(),
                    |ty| text(src, ty.type_annotation.span()),
                ),
                optional: property.optional,
            }),
            _ => None,
        })
        .collect()
}

fn string_literals(ty: &TSType) -> Option<Vec<String>> {
    let literal = |ty: &TSType| match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(text) => Some(text.value.to_string()),
            _ => None,
        },
        _ => None,
    };
    match ty {
        TSType::TSUnionType(union) => union.types.iter().map(literal).collect(),
        other => literal(other).map(|one| vec![one]),
    }
}

fn declaration(src: &str, declaration: &Declaration, exported: bool, module: &mut Module) {
    let line = line_of(src, declaration.span().start);
    let mut push = |name: &str, kind: Kind| {
        module.items.push(Item {
            name: name.to_string(),
            kind,
            exported,
            line,
        });
    };
    match declaration {
        Declaration::TSInterfaceDeclaration(interface) => {
            push(&interface.id.name, Kind::Interface);
            module.objects.push((
                interface.id.name.to_string(),
                fields(src, &interface.body.body),
            ));
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            push(&alias.id.name, Kind::TypeAlias);
            if let Some(members) = string_literals(&alias.type_annotation) {
                module.unions.push((alias.id.name.to_string(), members));
            }
            if let TSType::TSTypeLiteral(object) = &alias.type_annotation {
                module
                    .objects
                    .push((alias.id.name.to_string(), fields(src, &object.members)));
            }
        }
        Declaration::VariableDeclaration(variables) => {
            let kind = if variables.kind == oxc_ast::ast::VariableDeclarationKind::Const {
                Kind::Const
            } else {
                Kind::Variable
            };
            for declarator in &variables.declarations {
                if let BindingPatternKind::BindingIdentifier(ident) = &declarator.id.kind {
                    push(&ident.name, kind);
                }
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                push(&id.name, Kind::Function);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                push(&id.name, Kind::Class);
            }
        }
        _ => {}
    }
}

/// Reads what `src` declares, imports and exports.
///
/// # Errors
///
/// Returns the line and message of the first syntax error.
pub fn parse(src: &str) -> Result<Module, (usize, String)> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, src, SourceType::ts()).parse();
    if let Some(error) = parsed.errors.first() {
        let offset = error
            .labels
            .as_ref()
            .and_then(|labels| labels.first())
            .map_or(0, |label| u32::try_from(label.offset()).unwrap_or(0));
        return Err((line_of(src, offset), error.message.to_string()));
    }
    let mut module = Module::default();
    for statement in &parsed.program.body {
        match statement {
            Statement::ImportDeclaration(import) => {
                let names = import
                    .specifiers
                    .iter()
                    .flatten()
                    .map(|specifier| match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(one) => one.local.name,
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(one) => one.local.name,
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(one) => one.local.name,
                    })
                    .map(|name| name.to_string())
                    .collect();
                module.imports.push(Import {
                    source: import.source.value.to_string(),
                    type_only: import.import_kind == ImportOrExportKind::Type,
                    deferred: import.phase == Some(ImportPhase::Defer),
                    names,
                    line: line_of(src, import.span.start),
                });
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(inner) = &export.declaration {
                    declaration(src, inner, true, &mut module);
                }
                for specifier in &export.specifiers {
                    module.items.push(Item {
                        name: specifier.exported.name().to_string(),
                        kind: Kind::Other,
                        exported: true,
                        line: line_of(src, specifier.span.start),
                    });
                }
            }
            Statement::ExportDefaultDeclaration(export) => module.items.push(Item {
                name: "default".to_string(),
                kind: Kind::Other,
                exported: true,
                line: line_of(src, export.span.start),
            }),
            other => {
                if let Some(inner) = other.as_declaration() {
                    declaration(src, inner, false, &mut module);
                }
            }
        }
    }
    Ok(module)
}

/// What a walk needs to know about a node in order to wire it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeInfo {
    /// The node's name, `snake_case`.
    pub name: String,
    /// The edge names, `onSuccess`, in the order `Edge` lists them.
    pub edges: Vec<String>,
    /// The fields of its `State`.
    pub state: Vec<Field>,
    /// Whether it exports a `Utils` type to override in tests.
    pub utils: bool,
}

/// Reads a node's contract: `export type Edge`, `export interface State` (or `type State`),
/// `export type Utils` and `export const factory`.
///
/// # Errors
///
/// Returns a message naming what is missing or the syntax error.
pub fn node_info(name: &str, src: &str) -> Result<NodeInfo, String> {
    let module = parse(src).map_err(|(line, message)| format!("line {line}: {message}"))?;
    if module.exported("factory").is_none() {
        return Err("it has no `export const factory`".to_string());
    }
    let edges = module
        .exported("Edge")
        .and_then(|_| module.union("Edge"))
        .ok_or("it has no `export type Edge` of string literals")?
        .to_vec();
    if module.exported("State").is_none() {
        return Err("it has no `export interface State`".to_string());
    }
    Ok(NodeInfo {
        name: name.to_string(),
        edges,
        state: module
            .object("State")
            .map(<[Field]>::to_vec)
            .unwrap_or_default(),
        utils: module.exported("Utils").is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNT: &str = include_str!("../../../templates/ts/example/nodes/count.ts");
    const STOP: &str = include_str!("../../../templates/ts/example/nodes/stop.ts");
    const COUNTER: &str = include_str!("../../../templates/ts/example/walks/counter.ts");

    #[test]
    fn reads_a_node_contract() {
        let info = node_info("count", COUNT).unwrap();

        assert_eq!(info.edges, ["onCount", "onStop"]);
        assert_eq!(
            info.state,
            [Field {
                name: "count".to_string(),
                ty: "number".to_string(),
                optional: false
            }]
        );
        assert!(info.utils);
        assert_eq!(node_info("stop", STOP).unwrap().edges, ["onDone"]);
    }

    #[test]
    fn says_what_a_node_is_missing() {
        assert_eq!(
            node_info("x", "export const y = 1;\n").unwrap_err(),
            "it has no `export const factory`"
        );
        assert_eq!(
            node_info(
                "x",
                "export const factory = 1;\nexport type Edge = string;\n"
            )
            .unwrap_err(),
            "it has no `export type Edge` of string literals"
        );
        assert!(
            node_info("x", "export const = ;\n")
                .unwrap_err()
                .starts_with("line 1: ")
        );
    }

    #[test]
    fn reads_a_walk_module() {
        let module = parse(COUNTER).unwrap();

        assert_eq!(module.union("NodeId").unwrap(), ["START", "COUNT", "STOP"]);
        assert_eq!(module.exported("createWalk").unwrap().kind, Kind::Function);
        let deferred: Vec<&str> = module
            .imports
            .iter()
            .filter(|import| import.deferred)
            .map(|import| import.source.as_str())
            .collect();
        assert_eq!(
            deferred,
            ["../nodes/count.ts", "../nodes/start.ts", "../nodes/stop.ts"]
        );
        assert!(module.imports[0].names.contains(&"walk".to_string()));
        assert_eq!(module.object("WalkUtils").unwrap().len(), 3);
    }

    #[test]
    fn records_lines_and_private_items() {
        let module = parse("const a = 1;\n\nexport function b() {}\n").unwrap();
        assert_eq!(
            module.items,
            [
                Item {
                    name: "a".to_string(),
                    kind: Kind::Const,
                    exported: false,
                    line: 1
                },
                Item {
                    name: "b".to_string(),
                    kind: Kind::Function,
                    exported: true,
                    line: 3
                },
            ]
        );
    }
}
