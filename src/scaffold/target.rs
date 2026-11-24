//! The language a project is written in, and where its parts live.
//!
//! The target comes from `--target`, else the `AMBLE_TARGET` variable, else the files of the
//! project in the current folder, else it is Rust.
//!
//! A Python project's files live in its package, `src/<package>/`, so the Python target carries
//! the package's name, read from `pyproject.toml` once per run.

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
    /// ambler-py: a uv project whose package, `src/<package>/`, has `engine.py`.
    Python {
        /// The import name, `snake_case`. Empty until a project is known (`amble new`).
        package: &'static str,
    },
}

impl Target {
    /// Reads `rust`, `rs`, `typescript`, `ts`, `python` or `py`, in any case. Python's package is
    /// left empty for [`resolve`] to fill in.
    ///
    /// # Errors
    ///
    /// Returns a message listing the targets.
    pub fn parse(text: &str) -> Result<Self, String> {
        match text.trim().to_ascii_lowercase().as_str() {
            "rust" | "rs" => Ok(Self::Rust),
            "typescript" | "ts" => Ok(Self::TypeScript),
            "python" | "py" => Ok(Self::Python { package: "" }),
            _ => Err(format!(
                "`{text}` is not a target: use `rust`, `typescript` (`ts`) or `python` (`py`)"
            )),
        }
    }

    /// The name used in messages and in `--target`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Python { .. } => "python",
        }
    }

    /// Whether `self` and `other` are the same language, whatever Python's package.
    #[must_use]
    pub fn same_language(self, other: Self) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&other)
    }

    /// The command prefix that runs this target's native Ambler Tooling Protocol (ATP) CLI.
    #[must_use]
    pub fn native_runner(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Rust => ("cargo", &["ambler"]),
            Self::TypeScript => ("bun", &["run", "ambler"]),
            Self::Python { .. } => ("uv", &["run", "ambler-cli"]),
        }
    }

    /// Whether the project in the current folder can run its native CLI: `exists` tells which
    /// files it has and `installed` which programs are on the `PATH`.
    #[must_use]
    pub fn has_native_runner(
        self,
        exists: impl Fn(&str) -> bool,
        installed: impl Fn(&str) -> bool,
    ) -> bool {
        match self {
            Self::Rust => installed("cargo") && installed("cargo-ambler"),
            Self::TypeScript => {
                (exists("cli.ts") || exists("node_modules/.bin/ambler")) && installed("bun")
            }
            Self::Python { .. } => exists(".venv/bin/ambler-cli") && installed("uv"),
        }
    }

    /// The package runner that fetches the native tool when the project has not installed it:
    /// `uvx` for Python, `bunx` for TypeScript.
    #[must_use]
    pub fn global_runner(self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(("bunx", &["github:argenkiwi/ambler-ts"])),
            Self::Python { .. } => Some((
                "uvx",
                &[
                    "--from",
                    "git+https://github.com/argenkiwi/ambler-py",
                    "ambler-cli",
                ],
            )),
        }
    }

    /// The file a node lives in. `name` is `snake_case`.
    #[must_use]
    pub fn node_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/nodes/{name}.rs"),
            Self::TypeScript => format!("nodes/{}.ts", names::kebab(name)),
            Self::Python { package } => format!("src/{package}/nodes/{name}.py"),
        }
    }

    /// The file a node's tests live in, if not in the node's own file.
    #[must_use]
    pub fn node_test_path(self, name: &str) -> Option<String> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(format!("nodes/tests/{}.test.ts", names::kebab(name))),
            Self::Python { .. } => Some(format!("tests/nodes/test_{name}.py")),
        }
    }

    /// The file a walk lives in. `name` is `snake_case`.
    #[must_use]
    pub fn walk_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/walks/{name}.rs"),
            Self::TypeScript => format!("walks/{}.ts", names::kebab(name)),
            Self::Python { package } => format!("src/{package}/walks/{name}.py"),
        }
    }

    /// The file a walk's tests live in, if not in the walk's own file.
    #[must_use]
    pub fn walk_test_path(self, name: &str) -> Option<String> {
        match self {
            Self::Rust => None,
            Self::TypeScript => Some(format!("walks/tests/{}.test.ts", names::kebab(name))),
            Self::Python { .. } => Some(format!("tests/walks/test_{name}.py")),
        }
    }

    /// The file a utility module lives in. `name` is `snake_case`.
    #[must_use]
    pub fn util_path(self, name: &str) -> String {
        match self {
            Self::Rust => format!("src/utils/{name}.rs"),
            Self::TypeScript => format!("utils/{}.ts", names::kebab(name)),
            Self::Python { package } => format!("src/{package}/utils/{name}.py"),
        }
    }

    /// The engine, the same in every project of this target.
    #[must_use]
    pub fn engine(self) -> String {
        match self {
            Self::Rust => "src/engine.rs".to_string(),
            Self::TypeScript => "ambler.ts".to_string(),
            Self::Python { package } => format!("src/{package}/engine.py"),
        }
    }

    /// The folder that holds the nodes.
    #[must_use]
    pub fn nodes_folder(self) -> String {
        match self {
            Self::Rust => "src/nodes".to_string(),
            Self::TypeScript => "nodes".to_string(),
            Self::Python { package } => format!("src/{package}/nodes"),
        }
    }

    /// The text that marks code `amble node` and `amble walk` left unfinished.
    #[must_use]
    pub fn unfinished(self) -> &'static [&'static str] {
        match self {
            Self::Rust => &[concat!("todo", "!(")],
            Self::TypeScript => &[concat!("not yet ", "implemented"), concat!("unwired", "(")],
            Self::Python { .. } => &[
                concat!("not yet ", "implemented"),
                concat!("not yet ", "wired"),
            ],
        }
    }

    /// How to run one node's tests.
    #[must_use]
    pub fn node_test_command(self, name: &str) -> String {
        match self {
            Self::Rust => format!("cargo test nodes::{name}::"),
            Self::TypeScript => format!("bun test nodes/tests/{}.test.ts", names::kebab(name)),
            Self::Python { .. } => format!("uv run pytest tests/nodes/test_{name}.py"),
        }
    }

    /// Whether the current folder holds a project of this target: its manifest and its engine.
    #[must_use]
    pub fn is_project(self, exists: impl Fn(&str) -> bool) -> bool {
        let manifest = match self {
            Self::Rust => "Cargo.toml",
            Self::TypeScript => "package.json",
            Self::Python { package: "" } => return false,
            Self::Python { .. } => "pyproject.toml",
        };
        exists(manifest)
            && match self {
                Self::Python { package } => {
                    exists(&format!("src/{package}/engine.py"))
                        || exists(&format!("src/{package}/__init__.py"))
                }
                _ => exists(&self.engine()),
            }
    }

    /// The target whose marker files are in the current folder, if any. `read` gives a file's
    /// contents, or `None` if it is not there.
    #[must_use]
    pub fn detect(read: impl Fn(&str) -> Option<String>) -> Option<Self> {
        let exists = |path: &str| read(path).is_some();
        if Self::TypeScript.is_project(exists) {
            Some(Self::TypeScript)
        } else if Self::Rust.is_project(exists) {
            Some(Self::Rust)
        } else {
            let python = Self::Python {
                package: intern(python_package(&read)?),
            };
            python.is_project(exists).then_some(python)
        }
    }
}

/// The import name of the package `pyproject.toml` declares: its `[project]` name in
/// `snake_case`.
#[must_use]
pub fn python_package(read: impl Fn(&str) -> Option<String>) -> Option<String> {
    pyproject_name(&read("pyproject.toml")?).map(|name| package_of(&name))
}

/// The Python import name of a project named `name`: `my-app` is imported as `my_app`.
#[must_use]
pub fn package_of(name: &str) -> String {
    name.replace(['-', '.'], "_").to_ascii_lowercase()
}

/// The `name` of the `[project]` table of a `pyproject.toml`.
#[must_use]
pub fn pyproject_name(text: &str) -> Option<String> {
    let mut in_project = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_project = line == "[project]";
        } else if in_project
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "name"
        {
            let value = value.trim();
            let name = value
                .strip_prefix('"')
                .and_then(|value| value.split_once('"'))
                .or_else(|| {
                    value
                        .strip_prefix('\'')
                        .and_then(|value| value.split_once('\''))
                })?
                .0;
            return Some(name.to_string());
        }
    }
    None
}

/// A package name that lives as long as the run: there is one per invocation.
fn intern(package: String) -> &'static str {
    Box::leak(package.into_boxed_str())
}

/// Works out the target of an invocation of `verb`, and takes `--target` out of `args`.
///
/// `--target <t>` (or `--target=<t>`) comes first, then `env` (the value of [`VAR`]), then the
/// project's files, which `new` does not look at because it makes a project rather than
/// working in one; Rust is the default. A Python target outside `new` takes its package from
/// `pyproject.toml`.
///
/// # Errors
///
/// Returns a message if a target is not known or `--target` has no value.
pub fn resolve(
    verb: &str,
    args: Vec<String>,
    env: Option<String>,
    read: impl Fn(&str) -> Option<String>,
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
        (None, None) => Target::detect(&read).unwrap_or_default(),
    };
    let target = match target {
        Target::Python { package: "" } if verb != "new" => Target::Python {
            package: python_package(&read).map_or("", intern),
        },
        target => target,
    };
    Ok((target, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    fn ts_project(path: &str) -> Option<String> {
        matches!(path, "ambler.ts" | "package.json").then(String::new)
    }

    fn rust_project(path: &str) -> Option<String> {
        matches!(path, "Cargo.toml" | "src/engine.rs").then(String::new)
    }

    fn py_project(path: &str) -> Option<String> {
        match path {
            "pyproject.toml" => Some(
                "[build-system]\nname = \"x\"\n\n[project]\nversion = \"0.1.0\"\nname = \
                 \"my-app\"\n"
                    .to_string(),
            ),
            "src/my_app/engine.py" => Some(String::new()),
            _ => None,
        }
    }

    #[test]
    fn native_runner_returns_correct_tooling_commands() {
        assert_eq!(Target::Rust.native_runner(), ("cargo", &["ambler"][..]));
        assert_eq!(
            Target::TypeScript.native_runner(),
            ("bun", &["run", "ambler"][..])
        );
        assert_eq!(
            Target::Python { package: "foo" }.native_runner(),
            ("uv", &["run", "ambler-cli"][..])
        );
    }

    #[test]
    fn a_python_project_is_found_by_its_package_and_engine() {
        assert_eq!(
            resolve("check", vec![], None, py_project),
            Ok((Target::Python { package: "my_app" }, vec![]))
        );
        assert_eq!(
            resolve("node", args(&["--target", "py"]), None, py_project),
            Ok((Target::Python { package: "my_app" }, vec![]))
        );
        assert_eq!(
            resolve(
                "new",
                args(&["--target", "python", "app"]),
                None,
                py_project
            ),
            Ok((Target::Python { package: "" }, args(&["app"])))
        );
        let no_engine = |path: &str| py_project(path).filter(|_| path == "pyproject.toml");
        assert_eq!(
            resolve("check", vec![], None, no_engine),
            Ok((Target::Rust, vec![]))
        );
    }

    #[test]
    fn reads_the_project_name_of_a_pyproject() {
        assert_eq!(
            pyproject_name("[project]\nname = 'tool'\n").as_deref(),
            Some("tool")
        );
        assert_eq!(pyproject_name("[tool.x]\nname = \"no\"\n"), None);
        assert_eq!(package_of("My-App"), "my_app");
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
            resolve("check", vec![], None, |_| None),
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
            resolve("new", args(&["--target", "go"]), None, |_| None)
                .unwrap_err()
                .contains("`go` is not a target")
        );
        assert!(
            resolve("node", vec![], Some("go".to_string()), |_| None)
                .unwrap_err()
                .contains("`go` is not a target")
        );
        assert_eq!(
            resolve("new", args(&["--target"]), None, |_| None),
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
        let py = Target::Python { package: "app" };
        assert_eq!(py.node_path("check_input"), "src/app/nodes/check_input.py");
        assert_eq!(
            py.node_test_path("check_input").as_deref(),
            Some("tests/nodes/test_check_input.py")
        );
        assert_eq!(py.walk_path("my_walk"), "src/app/walks/my_walk.py");
        assert_eq!(
            py.walk_test_path("my_walk").as_deref(),
            Some("tests/walks/test_my_walk.py")
        );
        assert_eq!(py.util_path("llm_chat"), "src/app/utils/llm_chat.py");
        assert_eq!(py.engine(), "src/app/engine.py");
        assert!(py.same_language(Target::Python { package: "" }));
        assert!(!py.same_language(Target::Rust));
        assert_eq!(
            Target::TypeScript.util_path("llm_chat"),
            "utils/llm-chat.ts"
        );
    }
}
