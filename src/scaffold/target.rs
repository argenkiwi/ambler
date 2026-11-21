//! The language a project is written in, and where its parts live.
//!
//! The target comes from `--target`, else the `AMBLE_TARGET` variable, else the files of the
//! project in the current folder, else it is Rust.

use super::names;

/// The environment variable that names the target when `--target` is not given.
pub const VAR: &str = "AMBLE_TARGET";

/// A language amble generates and checks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    /// ambler-rs: a Cargo crate with `src/engine.rs`.
    #[default]
    Rust,
    /// ambler-ts: a Bun project with `ambler.ts`.
    TypeScript,
}

impl Target {
    /// Reads `rust`, `rs`, `typescript` or `ts`, in any case.
    ///
    /// # Errors
    ///
    /// Returns a message listing the targets.
    pub fn parse(text: &str) -> Result<Self, String> {
        match text.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "typescript" | "ts" => Ok(Self::TypeScript),
            _ => Err(format!(
                "`{text}` is not a target: use `rust` or `typescript` (`ts`)"
            )),
        }
    }

    /// The name used in messages and in `--target`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
        }
    }

    /// The file a node lives in. `name` is `snake_case`.
    #[must_use]
    pub fn node_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/nodes/{name}.rs"),
            Self::TypeScript => format!("nodes/{}.ts", names::kebab(name)),
        }
    }

    /// The file a node's tests live in, if not in the node's own file.
    #[must_use]
    pub fn node_test_path(self, name: &str) -> Option<String> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(format!("nodes/tests/{}.test.ts", names::kebab(name))),
        }
    }

    /// The file a walk lives in. `name` is `snake_case`.
    #[must_use]
    pub fn walk_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/walks/{name}.rs"),
            Self::TypeScript => format!("walks/{}.ts", names::kebab(name)),
        }
    }

    /// The file a walk's tests live in, if not in the walk's own file.
    #[must_use]
    pub fn walk_test_path(self, name: &str) -> Option<String> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(format!("walks/tests/{}.test.ts", names::kebab(name))),
        }
    }

    /// The file a utility module lives in. `name` is `snake_case`.
    #[must_use]
    pub fn util_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/utils/{name}.rs"),
            Self::TypeScript => format!("utils/{}.ts", names::kebab(name)),
        }
    }

    /// The engine, the same in every project of this target.
    #[must_use]
    pub fn engine(self) -> &'static str {
        match self {
            Self::Rust => "src/engine.rs",
            Self::TypeScript => "ambler.ts",
        }
    }

    /// The folder that holds the nodes.
    #[must_use]
    pub fn nodes_folder(self) -> &'static str {
        match self {
            Self::Rust => "src/nodes",
            Self::TypeScript => "nodes",
        }
    }

    /// The text that marks code `amble node` and `amble walk` left unfinished.
    #[must_use]
    pub fn unfinished(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[concat!("todo", "!(")],
            Self::TypeScript => &[concat!("not yet ", "implemented"), concat!("unwired", "(")],
        }
    }

    /// The commands a project has to pass, in order: program and arguments.
    #[must_use]
    pub fn checks(self) -> &'static [(&'static str, &'static [&'static str])] {
        match self {
            Self::Rust => &[
                ("cargo", &["fmt", "--check"]),
                (
                    "cargo",
                    &["clippy", "--all-targets", "--", "-D", "warnings"],
                ),
                ("cargo", &["test"]),
            ],
            Self::TypeScript => &[
                ("bun", &["run", "check"]),
                ("bun", &["run", "lint"]),
                ("bun", &["run", "test"]),
            ],
        }
    }

    /// How to run one node's tests.
    #[must_use]
    pub fn node_test_command(self, name: &str) -> String {
        match self {
            Self::Rust => format!("cargo test nodes::{name}::"),
            Self::TypeScript => format!("bun test nodes/tests/{}.test.ts", names::kebab(name)),
        }
    }

    /// The target whose marker files are in the current folder, if any.
    #[must_use]
    pub fn detect(exists: impl Fn(&str) -> bool) -> Option<Self> {
        if exists("ambler.ts") && exists("package.json") {
            Some(Self::TypeScript)
        } else if exists("Cargo.toml") && exists("src/engine.rs") {
            Some(Self::Rust)
        } else {
            None
        }
    }
}

/// Works out the target of an invocation of `verb`, and takes `--target` out of `args`.
///
/// `--target <t>` (or `--target=<t>`) comes first, then `env` (the value of [`VAR`]), then the
/// project's files, which `new` does not look at because it makes a project rather than
/// working in one; Rust is the default.
///
/// # Errors
///
/// Returns a message if a target is not known or `--target` has no value.
pub fn resolve(
    verb: &str,
    args: Vec<String>,
    env: Option<String>,
    exists: impl Fn(&str) -> bool,
) -> Result<(Target, Vec<String>), String> {
    let mut flag = None;
    let mut rest = Vec::with_capacity(args.len());
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--target" {
            let value = args.next().ok_or("`--target` needs a value")?;
            flag = Some(value);
        } else if let Some(value) = arg.strip_prefix("--target=") {
            flag = Some(value.to_string());
        } else {
            rest.push(arg);
        }
    }
    let target = match (flag, env.filter(|value| !value.trim().is_empty())) {
        (Some(value), _) | (None, Some(value)) => Target::parse(&value)?,
        (None, None) if verb == "new" => Target::default(),
        (None, None) => Target::detect(exists).unwrap_or_default(),
    };
    Ok((target, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    fn ts_project(path: &str) -> bool {
        matches!(path, "ambler.ts" | "package.json")
    }

    fn rust_project(path: &str) -> bool {
        matches!(path, "Cargo.toml" | "src/engine.rs")
    }

    #[test]
    fn the_flag_comes_first_and_is_taken_out_of_the_arguments() {
        let resolved = resolve(
            "node",
            args(&["ask", "--target", "ts", "--terminal"]),
            Some("rust".to_string()),
            rust_project,
        );
        assert_eq!(
            resolved,
            Ok((Target::TypeScript, args(&["ask", "--terminal"])))
        );

        let resolved = resolve("new", args(&["--target=rs", "app"]), None, ts_project);
        assert_eq!(resolved, Ok((Target::Rust, args(&["app"]))));
    }

    #[test]
    fn the_environment_comes_before_the_project_files() {
        let resolved = resolve(
            "check",
            vec![],
            Some("TypeScript".to_string()),
            rust_project,
        );
        assert_eq!(resolved, Ok((Target::TypeScript, vec![])));
    }

    #[test]
    fn the_project_files_decide_when_nothing_else_does() {
        assert_eq!(
            resolve("check", vec![], None, ts_project),
            Ok((Target::TypeScript, vec![]))
        );
        assert_eq!(
            resolve("check", vec![], Some(String::new()), rust_project),
            Ok((Target::Rust, vec![]))
        );
        assert_eq!(
            resolve("check", vec![], None, |_| false),
            Ok((Target::Rust, vec![]))
        );
    }

    #[test]
    fn new_does_not_look_at_the_folder_it_is_run_in() {
        assert_eq!(
            resolve("new", args(&["app"]), None, ts_project),
            Ok((Target::Rust, args(&["app"])))
        );
    }

    #[test]
    fn rejects_an_unknown_target_or_a_missing_value() {
        assert!(
            resolve("new", args(&["--target", "go"]), None, |_| false)
                .unwrap_err()
                .contains("`go` is not a target")
        );
        assert!(
            resolve("node", vec![], Some("go".to_string()), |_| false)
                .unwrap_err()
                .contains("`go` is not a target")
        );
        assert_eq!(
            resolve("new", args(&["--target"]), None, |_| false),
            Err("`--target` needs a value".to_string())
        );
    }

    #[test]
    fn places_each_part_where_the_target_expects_it() {
        assert_eq!(
            Target::Rust.node_path("check_input"),
            "src/nodes/check_input.rs"
        );
        assert_eq!(
            Target::TypeScript.node_path("check_input"),
            "nodes/check-input.ts"
        );
        assert_eq!(
            Target::TypeScript.node_test_path("check_input").as_deref(),
            Some("nodes/tests/check-input.test.ts")
        );
        assert_eq!(Target::Rust.node_test_path("check_input"), None);
        assert_eq!(Target::TypeScript.walk_path("my_walk"), "walks/my-walk.ts");
        assert_eq!(
            Target::TypeScript.util_path("llm_chat"),
            "utils/llm-chat.ts"
        );
    }
}
