//! Generates a Python node and its tests: `Outcome`, `Error`, `Utils`, `SystemUtils`, the node
//! class with a `run` that raises `not yet implemented`, and one failing test per outcome.
//!
//! The text is written the way `ruff format` and ruff's import sorting leave it, so a fresh node
//! passes `amble check`'s ruff steps before anything formats it.

use super::source::{Field, Module};
use crate::scaffold::names;

/// Ruff's default line length.
pub const WIDTH: usize = 88;

/// What `amble node` was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeSpec {
    /// `snake_case`.
    pub name: String,
    /// The import name of the project's package.
    pub package: String,
    /// The members of `Outcome`, `UPPER_SNAKE_CASE`; empty for a terminal node.
    pub outcomes: Vec<String>,
    pub fallible: bool,
    /// The state protocols `run` requires, from `nodes/__init__.py`: `HasCount`.
    pub protocols: Vec<String>,
    /// Their fields, for the test's `MockState`.
    pub state: Vec<Field>,
    /// The import statements the fields' types need.
    pub imports: Vec<String>,
}

impl NodeSpec {
    #[must_use]
    pub fn terminal(&self) -> bool {
        self.outcomes.is_empty()
    }
}

/// `success`, `invalid_input` or `InvalidInput` → `SUCCESS`, `INVALID_INPUT`.
#[must_use]
pub fn outcome_name(outcome: &str) -> String {
    names::shout(&names::snake(outcome))
}

/// Whether `name` is an `UPPER_SNAKE_CASE` member name.
#[must_use]
pub fn is_outcome_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// The protocol that holds `field`: `count` → `HasCount`.
#[must_use]
pub fn protocol_name(field: &str) -> String {
    format!("Has{}", names::pascal(field))
}

/// The value a test or a fresh state starts a field of type `ty` from.
#[must_use]
pub fn initial_value(ty: &str) -> String {
    let ty = ty.trim();
    let base = ty.split('[').next().unwrap_or(ty);
    if ty.split('|').any(|part| part.trim() == "None") {
        return "None".to_string();
    }
    match base {
        "int" => "0",
        "float" => "0.0",
        "str" => "\"\"",
        "bool" => "False",
        "bytes" => "b\"\"",
        "list" => "[]",
        "dict" => "{}",
        "set" => "set()",
        "tuple" => "()",
        _ => return format!("{ty}()"),
    }
    .to_string()
}

/// A dataclass field's default for type `ty`: a literal, or a factory for anything mutable. The
/// factory is the type itself, `list[str]` rather than `list`, which pyright's strict mode
/// would read as `list[Unknown]`.
#[must_use]
pub fn field_default(ty: &str) -> String {
    let value = initial_value(ty);
    match value.as_str() {
        "None" | "0" | "0.0" | "\"\"" | "False" | "b\"\"" | "()" => value,
        _ => format!("field(default_factory={})", ty.trim()),
    }
}

/// `head` + the items, comma-separated, + `tail`, laid out as ruff does: on one line if it fits,
/// else the items on one indented line, else one item a line with a trailing comma. `indent` is
/// the indentation of `head`'s line.
#[must_use]
pub fn bracket(indent: usize, head: &str, items: &[String], tail: &str) -> String {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 4);
    let flat = format!("{pad}{head}{}{tail}\n", items.join(", "));
    if flat.len() <= WIDTH + 1 || items.is_empty() {
        return flat;
    }
    let close = tail.chars().next().map(String::from).unwrap_or_default();
    let rest = &tail[close.len()..];
    let joined = format!("{inner}{}\n", items.join(", "));
    if joined.len() <= WIDTH + 1 {
        return format!("{pad}{head}\n{joined}{pad}{close}{rest}\n");
    }
    let mut out = format!("{pad}{head}\n");
    for item in items {
        out += &format!("{inner}{item},\n");
    }
    format!("{out}{pad}{close}{rest}\n")
}

/// `from <module> import <names>`, wrapped as ruff wraps it. `names` are sorted the way ruff's
/// isort does: constants, then classes, then the rest.
#[must_use]
pub fn from_import(module: &str, names: &[&str]) -> String {
    let mut sorted: Vec<&str> = names.to_vec();
    sorted.sort_by_key(|name| {
        let rank = if name.len() > 1 && name.chars().all(|c| !c.is_ascii_lowercase()) {
            0
        } else if name.starts_with(|c: char| c.is_ascii_uppercase()) {
            1
        } else {
            2
        };
        (rank, name.to_ascii_lowercase())
    });
    sorted.dedup();
    let flat = format!("from {module} import {}", sorted.join(", "));
    if flat.len() <= WIDTH {
        return flat + "\n";
    }
    let mut out = format!("from {module} import (\n");
    for name in sorted {
        out += &format!("    {name},\n");
    }
    out + ")\n"
}

/// The import blocks of a module: the standard library's, then the project's, sorted, with a
/// blank line between them. A statement for any other module goes with the standard library's;
/// `ruff check --fix` moves it if it belongs elsewhere.
#[must_use]
pub fn imports(package: &str, statements: &[String]) -> String {
    let key = |statement: &String| {
        let module = statement
            .trim_start_matches("from ")
            .trim_start_matches("import ")
            .split([' ', '\n'])
            .next()
            .unwrap_or_default()
            .to_string();
        (statement.starts_with("from "), module)
    };
    let ours = |statement: &&String| {
        let (_, module) = key(statement);
        module == package || module.starts_with(&format!("{package}."))
    };
    let mut blocks = Vec::new();
    for mine in [false, true] {
        let mut block: Vec<&String> = statements
            .iter()
            .filter(|statement| ours(statement) == mine)
            .collect();
        // Within a section, ruff puts `import x` before `from x import y`.
        block.sort_by_key(|statement| key(statement));
        block.dedup();
        if !block.is_empty() {
            blocks.push(
                block
                    .iter()
                    .map(|statement| statement.trim_end().to_string() + "\n")
                    .collect::<String>(),
            );
        }
    }
    blocks.join("\n")
}

fn mock_state(spec: &NodeSpec) -> String {
    if spec.state.is_empty() {
        return "@dataclass\nclass MockState:\n    \"\"\"The node needs no state.\"\"\"\n"
            .to_string();
    }
    let mut out = String::from("@dataclass\nclass MockState:\n");
    for field in &spec.state {
        out += &format!("    {}: {}\n", field.name, field.ty);
    }
    out
}

fn state_value(spec: &NodeSpec, head: &str) -> String {
    let items: Vec<String> = spec
        .state
        .iter()
        .map(|field| format!("{}={}", field.name, initial_value(&field.ty)))
        .collect();
    bracket(4, &format!("{head}MockState("), &items, ")")
}

/// The contents of `src/<package>/nodes/<name>.py` for a new node.
#[must_use]
pub fn node(spec: &NodeSpec) -> String {
    let class = names::pascal(&spec.name);
    let mut header = vec!["from dataclasses import dataclass, field".to_string()];
    if !spec.terminal() {
        header.push("from enum import Enum, auto".to_string());
    }
    header.push("from typing import Protocol".to_string());
    if !spec.protocols.is_empty() {
        let protocols: Vec<&str> = spec.protocols.iter().map(String::as_str).collect();
        header.push(from_import(&format!("{}.nodes", spec.package), &protocols));
    }
    let mut out = imports(&spec.package, &header);
    if !spec.terminal() {
        out += "\n\nclass Outcome(Enum):\n    \"\"\"Where this node leaves the walk.\"\"\"\n\n";
        for outcome in &spec.outcomes {
            out += &format!("    {outcome} = auto()\n    \"\"\"TODO: when this happens.\"\"\"\n");
        }
    }
    if spec.fallible {
        out += "\n\nclass Error(Exception):\n    \"\"\"TODO: what stops this node from finishing.\"\"\"\n";
    }
    let state = match spec.protocols.as_slice() {
        [] => "object".to_string(),
        [one] => one.clone(),
        many => {
            let mut bases = many.to_vec();
            bases.push("Protocol".to_string());
            out += "\n\n";
            out += &bracket(0, "class State(", &bases, "):");
            out += "    \"\"\"The state this node needs.\"\"\"\n";
            "State".to_string()
        }
    };
    let returns = if spec.terminal() { "None" } else { "Outcome" };
    out += &format!(
        "\n\n\
         class Utils(Protocol):\n    \
             \"\"\"The side effects this node needs: one method each, faked in the tests.\"\"\"\n\
         \n\
         \n\
         class SystemUtils:\n    \
             \"\"\"The real side effects.\"\"\"\n\
         \n\
         \n\
         @dataclass(frozen=True)\n\
         class {class}:\n    \
             \"\"\"TODO: one sentence on what this node does.\"\"\"\n\
         \n    \
             utils: Utils = field(default_factory=SystemUtils)\n\
         \n    \
             async def run(self, state: {state}) -> {returns}:\n        \
                 raise NotImplementedError(\"{marker}\")\n",
        marker = concat!("not yet ", "implemented"),
    );
    out
}

/// The contents of `tests/nodes/test_<name>.py`: one failing test per outcome, and one for the
/// error if the node can fail.
#[must_use]
pub fn tests(spec: &NodeSpec) -> String {
    let class = names::pascal(&spec.name);
    let mut header = vec!["from dataclasses import dataclass".to_string()];
    header.extend(spec.imports.iter().cloned());
    let mut names: Vec<&str> = vec![&class];
    if !spec.terminal() {
        names.push("Outcome");
    }
    if spec.fallible {
        names.push("Error");
    }
    let first = from_import(&format!("{}.nodes.{}", spec.package, spec.name), &names);
    header.push(first);
    let mut out = imports(&spec.package, &header);
    if spec.fallible {
        // pytest is a third-party module: its own section, between the other two.
        let (std, ours) = out.split_once("\n\n").unwrap_or((&out, ""));
        out = format!("{std}\n\nimport pytest\n\n{ours}");
    }
    out += &format!(
        "\n\n{}\n\nclass MockUtils:\n    \"\"\"Fakes every side effect of the node.\"\"\"\n",
        mock_state(spec)
    );
    let call = |indent: usize, head: &str| {
        bracket(
            indent,
            &format!("{head}await {class}(MockUtils()).run("),
            &["state".to_string()],
            ")",
        )
    };
    let start = state_value(spec, "state = ");
    let unchanged = state_value(spec, "assert state == ");
    if spec.terminal() {
        out += &format!(
            "\n\nasync def test_finishes_when_todo() -> None:\n{start}\n{}\n{unchanged}",
            call(4, "")
        );
    }
    for outcome in &spec.outcomes {
        out += &format!(
            "\n\nasync def test_{lower}_when_todo() -> None:\n{start}\n{call}\n    \
             assert outcome == Outcome.{outcome}\n{unchanged}",
            lower = outcome.to_ascii_lowercase(),
            call = call(4, "outcome = "),
        );
    }
    if spec.fallible {
        out += &format!(
            "\n\nasync def test_raises_error_when_todo() -> None:\n{start}\n    \
             with pytest.raises(Error):\n{}",
            call(8, "")
        );
    }
    out
}

/// `nodes/__init__.py` with a protocol for `field` of type `ty`, and `import` (a statement) if
/// the type needs one. `None` if the protocol is already there.
#[must_use]
pub fn add_protocol(
    init: &str,
    module: &Module,
    package: &str,
    field: &str,
    ty: &str,
    import: Option<&str>,
) -> Option<String> {
    let name = protocol_name(field);
    if module.class(&name).is_some() {
        return None;
    }
    let mut wanted: Vec<String> = Vec::new();
    if !module
        .imports
        .iter()
        .any(|import| import.module == "typing" && import.bound == "Protocol")
    {
        wanted.push("from typing import Protocol".to_string());
    }
    if let Some(import) = import
        && !module
            .imports
            .iter()
            .any(|known| known.statement() == import)
    {
        wanted.push(import.to_string());
    }
    let mut out = add_imports(init, module, package, &wanted);
    out = out.trim_end().to_string();
    out += &format!(
        "\n\n\nclass {name}(Protocol):\n    \"\"\"A state that has a {field} field.\"\"\"\n\n    \
         {field}: {ty}\n"
    );
    Some(out)
}

/// `src` with the import `statements` added: the whole import header is rewritten, sorted,
/// after the module docstring. Only single-line imports are moved; it is the generated
/// `nodes/__init__.py`'s style.
fn add_imports(src: &str, module: &Module, package: &str, statements: &[String]) -> String {
    if statements.is_empty() {
        return src.to_string();
    }
    let import_lines: Vec<usize> = module.imports.iter().map(|import| import.line).collect();
    let lines: Vec<&str> = src.lines().collect();
    let mut all: Vec<String> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if import_lines.contains(&(index + 1)) && !line.trim_end().ends_with('(') {
            all.push((*line).to_string());
        }
    }
    all.extend(statements.iter().cloned());
    let header = imports(package, &all);
    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            !(import_lines.contains(&(index + 1)) && !line.trim_end().ends_with('('))
        })
        .map(|(_, line)| *line)
        .collect();
    // The docstring (if any) ends where the first non-docstring statement could start.
    let docstring_end = docstring_lines(&kept);
    let before = kept[..docstring_end].join("\n");
    let after = kept[docstring_end..].join("\n");
    let after = after.trim();
    let mut out = String::new();
    if !before.trim().is_empty() {
        out += before.trim_end();
        out += "\n\n";
    }
    out += &header;
    if !after.is_empty() {
        out += "\n\n";
        out += after;
        out += "\n";
    }
    out
}

/// How many of `lines` the module docstring takes, if the module starts with one.
fn docstring_lines(lines: &[&str]) -> usize {
    let Some(first) = lines.first() else {
        return 0;
    };
    let first = first.trim_start();
    let Some(quote) = ["\"\"\"", "'''"]
        .into_iter()
        .find(|quote| first.starts_with(quote))
    else {
        return 0;
    };
    if first.len() >= 6 && first[3..].contains(quote) {
        return 1;
    }
    lines
        .iter()
        .skip(1)
        .position(|line| line.contains(quote))
        .map_or(lines.len(), |index| index + 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scaffold::py::source::parse;

    fn field(name: &str, ty: &str) -> Field {
        Field {
            name: name.to_string(),
            ty: ty.to_string(),
        }
    }

    fn spec(outcomes: &[&str], fallible: bool) -> NodeSpec {
        NodeSpec {
            name: "read_move".to_string(),
            package: "app".to_string(),
            outcomes: outcomes.iter().map(ToString::to_string).collect(),
            fallible,
            protocols: vec!["HasBoard".to_string()],
            state: vec![field("board", "list[str]")],
            imports: vec![],
        }
    }

    #[test]
    fn names_outcomes_in_capitals() {
        assert_eq!(outcome_name("success"), "SUCCESS");
        assert_eq!(outcome_name("invalid_input"), "INVALID_INPUT");
        assert_eq!(outcome_name("InvalidInput"), "INVALID_INPUT");
        assert!(is_outcome_name("INVALID_INPUT"));
        assert!(!is_outcome_name("Invalid"));
        assert_eq!(protocol_name("move_count"), "HasMoveCount");
    }

    #[test]
    fn starts_from_an_obvious_value_of_each_type() {
        assert_eq!(initial_value("int"), "0");
        assert_eq!(initial_value("list[str]"), "[]");
        assert_eq!(initial_value("Board | None"), "None");
        assert_eq!(initial_value("Board"), "Board()");
        assert_eq!(
            field_default("list[str]"),
            "field(default_factory=list[str])"
        );
        assert_eq!(field_default("str"), "\"\"");
        assert_eq!(field_default("Board"), "field(default_factory=Board)");
    }

    #[test]
    fn lays_out_brackets_as_ruff_does() {
        let short = ["a=0".to_string(), "b=1".to_string()];
        assert_eq!(bracket(4, "x = f(", &short, ")"), "    x = f(a=0, b=1)\n");
        let long: Vec<String> = (0..12).map(|i| format!("field_{i}=0")).collect();
        let laid = bracket(4, "state = MockState(", &long, ")");
        assert!(laid.starts_with("    state = MockState(\n        field_0=0,\n"));
        assert!(laid.ends_with("        field_11=0,\n    )\n"));
        assert_eq!(
            from_import("app.nodes.x", &["Outcome", "X", "Error"]),
            "from app.nodes.x import Error, Outcome, X\n"
        );
    }

    #[test]
    fn writes_the_contract_and_a_raising_run() {
        let source = node(&spec(&["VALID", "INVALID"], true));

        assert!(source.starts_with(
            "from dataclasses import dataclass, field\nfrom enum import Enum, auto\nfrom typing \
             import Protocol\n\nfrom app.nodes import HasBoard\n\n\nclass Outcome(Enum):\n"
        ));
        assert!(source.contains("    VALID = auto()\n"));
        assert!(source.contains("\n\n\nclass Error(Exception):\n"));
        assert!(source.contains("class ReadMove:\n"));
        assert!(source.contains("    async def run(self, state: HasBoard) -> Outcome:\n"));
        assert!(source.contains("raise NotImplementedError(\"not yet implemented\")\n"));
        assert!(parse(&source).is_ok());
    }

    #[test]
    fn a_terminal_node_with_several_protocols_has_a_local_state() {
        let mut spec = spec(&[], false);
        spec.protocols.push("HasTurn".to_string());

        let source = node(&spec);

        assert!(!source.contains("Enum"));
        assert!(source.contains("from app.nodes import HasBoard, HasTurn\n"));
        assert!(source.contains("\n\n\nclass State(HasBoard, HasTurn, Protocol):\n"));
        assert!(source.contains("async def run(self, state: State) -> None:\n"));
        assert!(parse(&source).is_ok());
    }

    #[test]
    fn tests_every_outcome_and_the_error() {
        let tests = tests(&spec(&["VALID", "INVALID"], true));

        assert!(tests.starts_with(
            "from dataclasses import dataclass\n\nimport pytest\n\nfrom app.nodes.read_move \
             import Error, Outcome, ReadMove\n\n\n@dataclass\nclass MockState:\n    board: \
             list[str]\n"
        ));
        assert!(tests.contains(
            "async def test_invalid_when_todo() -> None:\n    state = MockState(board=[])\n\n    \
             outcome = await ReadMove(MockUtils()).run(state)\n\n    assert outcome == \
             Outcome.INVALID\n    assert state == MockState(board=[])\n"
        ));
        assert!(tests.contains("    with pytest.raises(Error):\n        await ReadMove("));
        assert!(parse(&tests).is_ok());
    }

    #[test]
    fn a_terminal_node_is_tested_once() {
        let mut spec = spec(&[], false);
        spec.state.clear();

        let tests = tests(&spec);

        assert!(tests.contains("from app.nodes.read_move import ReadMove\n"));
        assert!(tests.contains("class MockState:\n    \"\"\"The node needs no state.\"\"\"\n"));
        assert!(tests.contains(
            "async def test_finishes_when_todo() -> None:\n    state = MockState()\n\n    await \
             ReadMove(MockUtils()).run(state)\n\n    assert state == MockState()\n"
        ));
        assert!(!tests.contains("pytest"));
    }

    #[test]
    fn adds_a_protocol_and_its_imports_to_the_nodes_package() {
        let init = "\"\"\"Nodes.\"\"\"\n";
        let added = add_protocol(
            init,
            &parse(init).unwrap(),
            "app",
            "board",
            "Board",
            Some("from app.board import Board"),
        )
        .unwrap();
        assert_eq!(
            added,
            "\"\"\"Nodes.\"\"\"\n\nfrom typing import Protocol\n\nfrom app.board import \
             Board\n\n\nclass HasBoard(Protocol):\n    \"\"\"A state that has a board \
             field.\"\"\"\n\n    board: Board\n"
        );

        let module = parse(&added).unwrap();
        let again = add_protocol(&added, &module, "app", "turn", "int", None).unwrap();
        assert!(again.starts_with("\"\"\"Nodes.\"\"\"\n\nfrom typing import Protocol\n\nfrom app"));
        assert!(again.ends_with(
            "    board: Board\n\n\nclass HasTurn(Protocol):\n    \"\"\"A state that has a turn \
             field.\"\"\"\n\n    turn: int\n"
        ));
        assert_eq!(
            add_protocol(&again, &parse(&again).unwrap(), "app", "turn", "int", None),
            None
        );
    }
}
