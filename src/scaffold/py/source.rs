//! Reads Python sources with rustpython-parser: what a module declares, a node's contract
//! (`Outcome`, `Error`, `Utils`, its class and the state `run` takes) and the state protocols of
//! `nodes/__init__.py`.

use crate::scaffold::names;
use rustpython_parser::ast::{self, Constant, Expr, Ranged, Stmt};
use rustpython_parser::text_size::TextRange;
use rustpython_parser::{Mode, parse as parse_python};

/// An annotated attribute of a class: `count: int`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    /// The annotation as written: `int`, `list[str]`, `Board | None`.
    pub ty: String,
}

/// A function or method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub is_async: bool,
    /// Each parameter and its annotation as written, `self` included.
    pub params: Vec<(String, Option<String>)>,
    /// The return annotation as written.
    pub returns: Option<String>,
    /// 1-based.
    pub line: usize,
}

/// A top-level class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Class {
    pub name: String,
    /// The bases as written: `Protocol`, `Walk[Id, State]`.
    pub bases: Vec<String>,
    /// The decorators as written, without `@`: `dataclass(frozen=True)`.
    pub decorators: Vec<String>,
    /// Annotated attributes.
    pub fields: Vec<Field>,
    /// Names assigned without an annotation: an enum's members.
    pub members: Vec<String>,
    pub methods: Vec<Function>,
    pub line: usize,
}

impl Class {
    #[must_use]
    pub fn method(&self, name: &str) -> Option<&Function> {
        self.methods.iter().find(|method| method.name == name)
    }

    #[must_use]
    pub fn has_base(&self, base: &str) -> bool {
        self.bases.iter().any(|known| known == base)
    }
}

/// One name an import binds: `from <module> import <name> as <bound>`, or `import <module>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    /// The module, with a leading `.` per level if relative.
    pub module: String,
    /// The imported name, or `None` for `import <module>`.
    pub name: Option<String>,
    /// The local name it binds.
    pub bound: String,
    pub line: usize,
}

impl Import {
    /// The import as one statement: `from x import Y` or `from x import Y as Z`.
    #[must_use]
    pub fn statement(&self) -> String {
        match &self.name {
            Some(name) if *name == self.bound => format!("from {} import {name}", self.module),
            Some(name) => format!("from {} import {name} as {}", self.module, self.bound),
            None if self.module == self.bound => format!("import {}", self.module),
            None => format!("import {} as {}", self.module, self.bound),
        }
    }
}

/// A top-level assignment: `Error = start.Error`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub name: String,
    /// The value as written.
    pub value: String,
    pub line: usize,
}

/// What a module declares at its top level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Module {
    pub classes: Vec<Class>,
    pub functions: Vec<Function>,
    pub assignments: Vec<Assignment>,
    pub imports: Vec<Import>,
    /// The string literals of `__all__`, if it is a list or tuple of them.
    pub all: Option<Vec<String>>,
}

impl Module {
    #[must_use]
    pub fn class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|class| class.name == name)
    }

    #[must_use]
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|function| function.name == name)
    }

    #[must_use]
    pub fn assignment(&self, name: &str) -> Option<&Assignment> {
        self.assignments
            .iter()
            .find(|assignment| assignment.name == name)
    }

    /// Whether the module defines `name` at its top level, by any means.
    #[must_use]
    pub fn defines(&self, name: &str) -> bool {
        self.class(name).is_some()
            || self.function(name).is_some()
            || self.assignment(name).is_some()
            || self.imports.iter().any(|import| import.bound == name)
    }

    /// The state protocols: classes deriving from `Protocol`, with the fields of every protocol
    /// they derive from, inherited first.
    #[must_use]
    pub fn protocol_fields(&self, name: &str) -> Option<Vec<Field>> {
        let class = self
            .class(name)
            .filter(|class| class.has_base("Protocol"))?;
        let mut fields: Vec<Field> = Vec::new();
        for base in &class.bases {
            for field in self.protocol_fields(base).unwrap_or_default() {
                if fields.iter().all(|known| known.name != field.name) {
                    fields.push(field);
                }
            }
        }
        for field in &class.fields {
            if fields.iter().all(|known| known.name != field.name) {
                fields.push(field.clone());
            }
        }
        Some(fields)
    }

    /// The imports that bind a name `ty` mentions, as statements: what a module needs to spell
    /// `ty` the way this one does.
    #[must_use]
    pub fn imports_for(&self, ty: &str) -> Vec<String> {
        let words: Vec<&str> = ty
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .collect();
        self.imports
            .iter()
            .filter(|import| words.contains(&import.bound.as_str()))
            .map(Import::statement)
            .collect()
    }
}

/// The 1-based line of byte `offset` in `src`.
#[must_use]
pub fn line_of(src: &str, offset: usize) -> usize {
    src[..offset.min(src.len())].matches('\n').count() + 1
}

struct Reader<'a> {
    src: &'a str,
}

impl Reader<'_> {
    fn text(&self, range: TextRange) -> String {
        self.src[range].to_string()
    }

    fn line(&self, range: TextRange) -> usize {
        line_of(self.src, range.start().to_usize())
    }

    fn function(
        &self,
        name: &str,
        is_async: bool,
        args: &ast::Arguments,
        returns: Option<&Expr>,
        range: TextRange,
    ) -> Function {
        let params = args
            .posonlyargs
            .iter()
            .chain(&args.args)
            .chain(&args.kwonlyargs)
            .map(|arg| {
                (
                    arg.def.arg.to_string(),
                    arg.def.annotation.as_ref().map(|ty| self.text(ty.range())),
                )
            })
            .collect();
        Function {
            name: name.to_string(),
            is_async,
            params,
            returns: returns.map(|ty| self.text(ty.range())),
            line: self.line(range),
        }
    }

    fn class(&self, class: &ast::StmtClassDef) -> Class {
        let mut out = Class {
            name: class.name.to_string(),
            bases: class
                .bases
                .iter()
                .map(|base| self.text(base.range()))
                .collect(),
            decorators: class
                .decorator_list
                .iter()
                .map(|decorator| self.text(decorator.range()))
                .collect(),
            fields: Vec::new(),
            members: Vec::new(),
            methods: Vec::new(),
            line: self.line(class.range),
        };
        for stmt in &class.body {
            match stmt {
                Stmt::AnnAssign(assign) => {
                    if let Expr::Name(target) = assign.target.as_ref() {
                        out.fields.push(Field {
                            name: target.id.to_string(),
                            ty: self.text(assign.annotation.range()),
                        });
                    }
                }
                Stmt::Assign(assign) => {
                    for target in &assign.targets {
                        if let Expr::Name(target) = target {
                            out.members.push(target.id.to_string());
                        }
                    }
                }
                Stmt::FunctionDef(def) => out.methods.push(self.function(
                    &def.name,
                    false,
                    &def.args,
                    def.returns.as_deref(),
                    def.range,
                )),
                Stmt::AsyncFunctionDef(def) => out.methods.push(self.function(
                    &def.name,
                    true,
                    &def.args,
                    def.returns.as_deref(),
                    def.range,
                )),
                _ => {}
            }
        }
        out
    }

    fn strings(value: &Expr) -> Option<Vec<String>> {
        let elements = match value {
            Expr::List(list) => &list.elts,
            Expr::Tuple(tuple) => &tuple.elts,
            _ => return None,
        };
        elements
            .iter()
            .map(|element| match element {
                Expr::Constant(ast::ExprConstant {
                    value: Constant::Str(text),
                    ..
                }) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    fn module(&self, body: &[Stmt]) -> Module {
        let mut module = Module::default();
        for stmt in body {
            match stmt {
                Stmt::ClassDef(class) => module.classes.push(self.class(class)),
                Stmt::FunctionDef(def) => module.functions.push(self.function(
                    &def.name,
                    false,
                    &def.args,
                    def.returns.as_deref(),
                    def.range,
                )),
                Stmt::AsyncFunctionDef(def) => module.functions.push(self.function(
                    &def.name,
                    true,
                    &def.args,
                    def.returns.as_deref(),
                    def.range,
                )),
                Stmt::Assign(assign) => {
                    for target in &assign.targets {
                        let Expr::Name(target) = target else {
                            continue;
                        };
                        if target.id.as_str() == "__all__" {
                            module.all = Self::strings(&assign.value);
                        }
                        module.assignments.push(Assignment {
                            name: target.id.to_string(),
                            value: self.text(assign.value.range()),
                            line: self.line(assign.range),
                        });
                    }
                }
                Stmt::AnnAssign(assign) => {
                    if let (Expr::Name(target), Some(value)) =
                        (assign.target.as_ref(), assign.value.as_ref())
                    {
                        module.assignments.push(Assignment {
                            name: target.id.to_string(),
                            value: self.text(value.range()),
                            line: self.line(assign.range),
                        });
                    }
                }
                Stmt::Import(import) => {
                    for alias in &import.names {
                        let bound = alias.asname.as_ref().map_or_else(
                            || alias.name.split('.').next().unwrap_or_default().to_string(),
                            ToString::to_string,
                        );
                        module.imports.push(Import {
                            module: alias.name.to_string(),
                            name: None,
                            bound,
                            line: self.line(import.range),
                        });
                    }
                }
                Stmt::ImportFrom(import) => {
                    let level = import.level.as_ref().map_or(0, ast::Int::to_usize);
                    let from = format!(
                        "{}{}",
                        ".".repeat(level),
                        import.module.as_ref().map_or("", |module| module.as_str())
                    );
                    for alias in &import.names {
                        module.imports.push(Import {
                            module: from.clone(),
                            name: Some(alias.name.to_string()),
                            bound: alias.asname.as_ref().unwrap_or(&alias.name).to_string(),
                            line: self.line(import.range),
                        });
                    }
                }
                _ => {}
            }
        }
        module
    }
}

/// Parses `src` as a Python module.
///
/// # Errors
///
/// Returns the line and message of the first syntax error.
pub fn parse(src: &str) -> Result<Module, (usize, String)> {
    let parsed = parse_python(src, Mode::Module, "<amble>").map_err(|error| {
        (
            line_of(src, error.offset.to_usize()),
            error.error.to_string(),
        )
    })?;
    let ast::Mod::Module(module) = parsed else {
        return Ok(Module::default());
    };
    Ok(Reader { src }.module(&module.body))
}

/// What `amble walk` needs to know about a node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeInfo {
    /// `snake_case`, the module's name.
    pub name: String,
    /// The members of `Outcome`, or `None` for a terminal node.
    pub outcomes: Option<Vec<String>>,
    /// Whether the module defines `class Error(Exception)`.
    pub fallible: bool,
    /// Whether `Utils` has any methods.
    pub utils: bool,
    /// The fields of the state `run` takes.
    pub state: Vec<Field>,
    /// The import statements the state's field types need, from `nodes/__init__.py`.
    pub imports: Vec<String>,
}

/// Reads the node `name` from its source, with `init` (the parsed `nodes/__init__.py`) for the
/// state protocols.
///
/// # Errors
///
/// Returns a message naming what the node lacks.
pub fn node_info(name: &str, src: &str, init: &Module) -> Result<NodeInfo, String> {
    let module = parse(src).map_err(|(line, error)| format!("line {line}: {error}"))?;
    let pascal = names::pascal(name);
    let class = module
        .class(&pascal)
        .ok_or_else(|| format!("it has no class `{pascal}`"))?;
    let run = class
        .method("run")
        .filter(|run| run.is_async)
        .ok_or_else(|| format!("`{pascal}` has no `async def run(self, state)`"))?;
    let annotation = run
        .params
        .get(1)
        .and_then(|(_, ty)| ty.clone())
        .ok_or_else(|| format!("`{pascal}.run` does not annotate its state parameter"))?;
    let state = if annotation == "object" {
        Vec::new()
    } else if let Some(local) = module.class(&annotation).filter(|c| c.has_base("Protocol")) {
        let mut fields: Vec<Field> = Vec::new();
        for base in &local.bases {
            for field in init.protocol_fields(base).unwrap_or_default() {
                if fields.iter().all(|known| known.name != field.name) {
                    fields.push(field);
                }
            }
        }
        for field in &local.fields {
            if fields.iter().all(|known| known.name != field.name) {
                fields.push(field.clone());
            }
        }
        fields
    } else {
        init.protocol_fields(&annotation).ok_or_else(|| {
            format!("`{annotation}`, the state of `{pascal}.run`, is not a protocol in nodes/__init__.py")
        })?
    };
    let mut imports: Vec<String> = Vec::new();
    for field in &state {
        for import in init.imports_for(&field.ty) {
            if !imports.contains(&import) {
                imports.push(import);
            }
        }
    }
    let outcomes = module
        .class("Outcome")
        .filter(|outcome| outcome.has_base("Enum"))
        .map(|outcome| outcome.members.clone());
    if outcomes.is_none() && run.returns.as_deref() != Some("None") {
        return Err(format!(
            "it has no `class Outcome(Enum)`, and `{pascal}.run` does not return `None`"
        ));
    }
    Ok(NodeInfo {
        name: name.to_string(),
        outcomes,
        fallible: module
            .class("Error")
            .is_some_and(|error| !error.bases.is_empty()),
        utils: module
            .class("Utils")
            .is_some_and(|utils| !utils.methods.is_empty()),
        state,
        imports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::scaffold::py::project::{ENGINE, example};

    /// An example file, in the package `app`.
    fn file(path: &str) -> String {
        example("app")
            .into_iter()
            .find(|(known, _)| known == path)
            .unwrap_or_else(|| panic!("no {path} in the example"))
            .1
    }

    #[test]
    fn reads_the_engine_with_its_generic_classes() {
        let module = parse(ENGINE).unwrap();

        assert_eq!(
            module.all.as_deref(),
            Some(&["Node".to_string(), "Walk".to_string()][..])
        );
        let walk = module.class("Walk").unwrap();
        assert_eq!(walk.bases, ["ABC"]);
        assert!(walk.method("step").unwrap().is_async);
    }

    #[test]
    fn reports_the_line_of_a_syntax_error() {
        let (line, _) = parse("x = 1\n\ndef broken(:\n").unwrap_err();
        assert_eq!(line, 3);
    }

    #[test]
    fn reads_a_node_its_outcomes_error_utils_and_state() {
        let init = parse(&file("src/app/nodes/__init__.py")).unwrap();

        let start = node_info("start", &file("src/app/nodes/start.py"), &init).unwrap();

        assert_eq!(
            start.outcomes.as_deref(),
            Some(&["SUCCESS".to_string(), "INVALID_INPUT".to_string()][..])
        );
        assert!(start.fallible);
        assert!(start.utils);
        assert_eq!(
            start.state,
            [Field {
                name: "count".to_string(),
                ty: "int".to_string()
            }]
        );

        let stop = node_info("stop", &file("src/app/nodes/stop.py"), &init).unwrap();
        assert_eq!(stop.outcomes, None);
        assert!(!stop.fallible);
    }

    #[test]
    fn a_node_local_state_combines_protocols_and_their_imports() {
        let init = parse(
            "from typing import Protocol\n\nfrom app.board import Board\n\n\n\
             class HasBoard(Protocol):\n    board: Board\n\n\n\
             class HasTurn(Protocol):\n    turn: int\n",
        )
        .unwrap();
        let src = "class State(HasBoard, HasTurn, Protocol):\n    \"\"\"x\"\"\"\n\n\n\
                   class Outcome(Enum):\n    A = auto()\n\n\n\
                   class Play:\n    async def run(self, state: State) -> Outcome: ...\n";

        let play = node_info("play", src, &init).unwrap();

        let names: Vec<&str> = play.state.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["board", "turn"]);
        assert_eq!(play.imports, ["from app.board import Board"]);
        assert!(!play.fallible);
        assert!(!play.utils);
    }

    #[test]
    fn says_what_a_node_lacks() {
        let init = Module::default();
        assert!(
            node_info("go", "x = 1\n", &init)
                .unwrap_err()
                .contains("no class `Go`")
        );
        assert!(
            node_info(
                "go",
                "class Go:\n    async def run(self, state: HasX) -> None: ...\n",
                &init
            )
            .unwrap_err()
            .contains("not a protocol")
        );
        assert!(
            node_info(
                "go",
                "class Go:\n    async def run(self, state: object) -> int: ...\n",
                &init
            )
            .unwrap_err()
            .contains("no `class Outcome(Enum)`")
        );
    }
}
