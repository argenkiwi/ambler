//! `ambler new`: creates a project. See `specs/new.md`.

use super::State;
use crate::nodes::plan_project::{self, PlanProject};
use crate::nodes::report::{self, Report};
use crate::nodes::write::{self, Write};
use crate::scaffold::target::Target;
use crate::{Node, Transition, Walk};

/// A rejected invocation, or a file that could not be written.
pub type Error = super::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Id {
    PlanProject,
    Write,
    Report,
}

#[derive(Default)]
struct NewProject<A = plan_project::SystemUtils, B = write::SystemUtils, C = report::SystemUtils> {
    plan_project: PlanProject<A>,
    write: Write<B>,
    report: Report<C>,
}

impl NewProject {
    /// Production wiring: every node uses its `SystemUtils`.
    fn new() -> Self {
        Self::default()
    }
}

impl<A, B, C> Walk for NewProject<A, B, C>
where
    A: plan_project::Utils + Sync,
    B: write::Utils + Sync,
    C: report::Utils + Sync,
{
    type Id = Id;
    type State = State;
    type Error = Error;

    async fn step(&self, id: Id, state: &mut State) -> Result<Transition<Id>, Error> {
        Ok(match id {
            Id::PlanProject => {
                let Ok(outcome) = self.plan_project.run(state).await;
                match outcome {
                    plan_project::Outcome::Planned => Transition::Next(Id::Write),
                    plan_project::Outcome::Rejected => Transition::Next(Id::Report),
                }
            }
            Id::Write => {
                self.write.run(state).await?;
                Transition::Halt
            }
            Id::Report => {
                let Ok(()) = self.report.run(state).await;
                Transition::Halt
            }
        })
    }
}

/// Runs `ambler new [<name>] [--example]`.
///
/// # Errors
///
/// Returns [`Error`] if the invocation is rejected (the reason is already on standard error) or
/// a file cannot be written.
pub async fn run(target: Target, args: Vec<String>) -> Result<(), Error> {
    let mut state = State::new(target, args);
    NewProject::new().run(Id::PlanProject, &mut state).await?;
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Fake;

    async fn new(fake: &Fake, args: &[&str]) -> State {
        let walk = NewProject {
            plan_project: PlanProject::with_utils(fake),
            write: Write::with_utils(fake),
            report: Report::with_utils(fake),
        };
        let mut state = State::with_args(args);
        walk.run(Id::PlanProject, &mut state).await.unwrap();
        state
    }

    #[tokio::test]
    async fn writes_the_project_and_says_what_to_do_next() {
        let fake = Fake::default();

        let state = new(&fake, &["my-app", "--example"]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("my-app/src/engine.rs")
                .unwrap()
                .contains("pub trait Node")
        );
        assert!(fake.file("my-app/src/walks/counter.rs").is_some());
        assert_eq!(
            fake.file("my-app/.claude/skills").as_deref(),
            Some("-> ../.agents/skills")
        );
        assert!(
            fake.printed()
                .last()
                .unwrap()
                .starts_with("Created my-app/.")
        );
        assert!(fake.eprinted().is_empty());
    }

    #[tokio::test]
    async fn without_a_name_writes_the_project_in_the_empty_current_folder() {
        let fake = Fake {
            folder: Some("my-app"),
            ..Fake::default()
        };

        let state = new(&fake, &[]).await;

        assert_eq!(state.message, None);
        assert!(
            fake.file("src/engine.rs")
                .unwrap()
                .contains("pub trait Node")
        );
        assert_eq!(
            fake.file(".claude/skills").as_deref(),
            Some("-> ../.agents/skills")
        );
        assert!(
            fake.printed()
                .last()
                .unwrap()
                .starts_with("Created my-app here.")
        );
    }

    #[tokio::test]
    async fn reports_a_rejected_invocation_and_writes_nothing() {
        let fake = Fake::default();

        let state = new(&fake, &["a", "b"]).await;

        assert!(state.finish().is_err());
        assert_eq!(
            fake.eprinted(),
            ["usage: ambler new [<name>] [--example] [--target rust|ts|py]"]
        );
        assert!(fake.paths().is_empty());
        assert!(fake.printed().is_empty());
    }

    /// End to end against the real file system and cargo: the example project `ambler new`
    /// writes builds, passes its tests and passes `ambler check`.
    #[tokio::test]
    #[ignore = "slow: builds a generated project with cargo"]
    async fn the_generated_example_project_passes_its_own_checks() {
        use std::process::Command;

        let folder = std::env::temp_dir().join(format!("ambler-e2e-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let ambler = |args: &[&str], within: &std::path::Path| {
            let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
            let status = Command::new("cargo")
                .args(["run", "--quiet", "--manifest-path", manifest, "--"])
                .args(args)
                .current_dir(within)
                .status()
                .unwrap();
            assert!(status.success(), "ambler {args:?} failed");
        };

        ambler(&["new", "demo", "--example"], &folder);
        ambler(&["check"], &folder.join("demo"));

        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// End to end against the real file system and Bun: a TypeScript example project passes
    /// `ambler check` without `--target`, and a node and walk added with ambler pass it once
    /// written.
    #[tokio::test]
    #[ignore = "slow: installs and checks a generated project with bun"]
    async fn a_generated_typescript_project_passes_its_own_checks() {
        use std::path::Path;
        use std::process::Command;

        let folder = std::env::temp_dir().join(format!("ambler-e2e-ts-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
        let ambler = |args: &[&str], within: &Path| {
            Command::new("cargo")
                .args(["run", "--quiet", "--manifest-path", manifest, "--"])
                .args(args)
                .current_dir(within)
                .env_remove(crate::scaffold::target::VAR)
                .status()
                .unwrap()
                .success()
        };
        let demo = folder.join("demo");
        let write = |path: &str, contents: &str| std::fs::write(demo.join(path), contents).unwrap();

        assert!(ambler(
            &["new", "demo", "--example", "--target", "ts"],
            &folder
        ));
        let installed = Command::new("bun")
            .arg("install")
            .current_dir(&demo)
            .status()
            .unwrap();
        assert!(installed.success());
        assert!(ambler(&["check"], &demo), "the example fails its checks");

        assert!(ambler(
            &[
                "node",
                "greet",
                "--outcomes",
                "done",
                "--state",
                "name:string"
            ],
            &demo
        ));
        assert!(ambler(
            &["node", "bye", "--terminal", "--state", "name"],
            &demo
        ));
        assert!(ambler(
            &[
                "walk",
                "hello",
                "--nodes",
                "greet,bye",
                "--edge",
                "greet.done=bye"
            ],
            &demo
        ));
        assert!(
            !ambler(&["check", "--lint-only"], &demo),
            "unfinished code passes"
        );

        write(
            "nodes/greet.ts",
            "import type { NodeFactory } from \"../ambler.ts\";\n\n\
             export interface State {\n  name: string;\n}\n\n\
             export type Edge = \"onDone\";\n\n\
             export type Utils = Record<string, never>;\n\n\
             const defaultUtils: Utils = {};\n\n\
             /** Names whoever is greeted. */\n\
             export const factory: NodeFactory<State, Edge, Utils> =\n  \
               (edges, _utils = defaultUtils) =>\n  \
               (state) => [edges.onDone, { ...state, name: \"world\" }];\n",
        );
        write(
            "nodes/tests/greet.test.ts",
            "import { expect, test } from \"bun:test\";\n\
             import { factory } from \"../greet.ts\";\n\n\
             test(\"greet takes onDone with a name\", async () => {\n  \
               const [next, after] = await factory({ onDone: \"DONE\" }, {})({ name: \"\" });\n\n  \
               expect(next).toEqual(\"DONE\");\n  \
               expect(after.name).toEqual(\"world\");\n\
             });\n",
        );
        write(
            "nodes/bye.ts",
            "import type { NodeFactory } from \"../ambler.ts\";\n\n\
             export interface State {\n  name: string;\n}\n\n\
             export type Edge = \"onDone\";\n\n\
             export type Utils = { print: (message: string) => void };\n\n\
             const defaultUtils: Utils = { print: (message) => console.log(message) };\n\n\
             /** Says goodbye. */\n\
             export const factory: NodeFactory<State, Edge, Utils> =\n  \
               (edges, utils = defaultUtils) =>\n  \
               (state) => {\n    \
                 utils.print(`Bye, ${state.name}`);\n    \
                 return [edges.onDone, state];\n  \
               };\n",
        );
        write(
            "nodes/tests/bye.test.ts",
            "import { expect, test } from \"bun:test\";\n\
             import { factory } from \"../bye.ts\";\n\n\
             test(\"bye takes onDone and says goodbye\", async () => {\n  \
               const printed: string[] = [];\n  \
               const print = (message: string) => {\n    printed.push(message);\n  };\n\n  \
               const [next] = await factory({ onDone: null }, { print })({ name: \"Ann\" });\n\n  \
               expect(next).toBeNull();\n  \
               expect(printed).toEqual([\"Bye, Ann\"]);\n\
             });\n",
        );
        write(
            "walks/tests/hello.test.ts",
            "import { expect, test } from \"bun:test\";\n\
             import { walk } from \"../../ambler.ts\";\n\
             import { createWalk } from \"../hello.ts\";\n\n\
             test(\"hello greets the world and says goodbye\", async () => {\n  \
               const printed: string[] = [];\n  \
               const print = (message: string) => {\n    printed.push(message);\n  };\n\n  \
               const final = await walk(createWalk({ bye: { print } }), \"GREET\", {\n    \
                 name: \"\",\n  \
               });\n\n  \
               expect(final.name).toEqual(\"world\");\n  \
               expect(printed).toEqual([\"Bye, world\"]);\n\
             });\n",
        );
        let biome = Command::new("node_modules/.bin/biome")
            .args(["check", "--write", "nodes", "walks"])
            .current_dir(&demo)
            .status()
            .unwrap();
        assert!(biome.success());
        assert!(ambler(&["check"], &demo), "the finished node and walk fail");

        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// End to end against the real file system and uv: a Python example project passes `ambler
    /// check` (ruff, pyright, pytest) without `--target`, generated nodes fail their tests with
    /// `not yet implemented`, and a node and walk added with ambler pass once written.
    #[tokio::test]
    #[ignore = "slow: syncs and checks a generated project with uv"]
    async fn a_generated_python_project_passes_its_own_checks() {
        use std::path::Path;
        use std::process::Command;

        let folder = std::env::temp_dir().join(format!("ambler-e2e-py-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
        let ambler = |args: &[&str], within: &Path| {
            Command::new("cargo")
                .args(["run", "--quiet", "--manifest-path", manifest, "--"])
                .args(args)
                .current_dir(within)
                .env_remove(crate::scaffold::target::VAR)
                .status()
                .unwrap()
                .success()
        };
        let demo = folder.join("demo-app");
        let uv = |args: &[&str]| {
            Command::new("uv")
                .args(args)
                .current_dir(&demo)
                .output()
                .unwrap()
        };
        let write = |path: &str, contents: &str| std::fs::write(demo.join(path), contents).unwrap();

        assert!(ambler(
            &["new", "demo-app", "--example", "--target", "py"],
            &folder
        ));
        assert!(uv(&["sync"]).status.success());
        assert!(ambler(&["check"], &demo), "the example fails its checks");

        assert!(ambler(
            &[
                "node",
                "greet",
                "--outcomes",
                "done",
                "--fallible",
                "--state",
                "name:str"
            ],
            &demo
        ));
        assert!(ambler(
            &["node", "bye", "--terminal", "--state", "name"],
            &demo
        ));
        assert!(ambler(
            &[
                "walk",
                "hello",
                "--nodes",
                "greet,bye",
                "--edge",
                "greet.done=bye"
            ],
            &demo
        ));
        assert!(
            !ambler(&["check", "--lint-only"], &demo),
            "unfinished code passes"
        );
        let red = uv(&["run", "pytest", "tests/nodes/test_greet.py"]);
        assert!(!red.status.success());
        assert!(String::from_utf8_lossy(&red.stdout).contains("not yet implemented"));

        write(
            "src/demo_app/nodes/greet.py",
            "from dataclasses import dataclass, field\n\
             from enum import Enum, auto\n\
             from typing import Protocol\n\n\
             from demo_app.nodes import HasName\n\n\n\
             class Outcome(Enum):\n    \"\"\"Where this node leaves the walk.\"\"\"\n\n    \
                 DONE = auto()\n    \"\"\"Someone was named.\"\"\"\n\n\n\
             class Error(Exception):\n    \"\"\"Nobody could be named.\"\"\"\n\n\n\
             class Utils(Protocol):\n    \"\"\"Nothing.\"\"\"\n\n\n\
             class SystemUtils:\n    \"\"\"Nothing.\"\"\"\n\n\n\
             @dataclass(frozen=True)\n\
             class Greet:\n    \"\"\"Names whoever is greeted.\"\"\"\n\n    \
                 utils: Utils = field(default_factory=SystemUtils)\n\n    \
                 async def run(self, state: HasName) -> Outcome:\n        \
                     state.name = \"world\"\n        \
                     return Outcome.DONE\n",
        );
        write(
            "tests/nodes/test_greet.py",
            "from dataclasses import dataclass\n\n\
             from demo_app.nodes.greet import Greet, Outcome\n\n\n\
             @dataclass\nclass MockState:\n    name: str\n\n\n\
             class MockUtils:\n    \"\"\"Nothing.\"\"\"\n\n\n\
             async def test_names_the_world() -> None:\n    \
                 state = MockState(name=\"\")\n\n    \
                 assert await Greet(MockUtils()).run(state) == Outcome.DONE\n    \
                 assert state.name == \"world\"\n",
        );
        write(
            "src/demo_app/nodes/bye.py",
            "from dataclasses import dataclass, field\n\
             from typing import Protocol\n\n\
             from demo_app.nodes import HasName\n\n\n\
             class Utils(Protocol):\n    def print(self, message: str) -> None: ...\n\n\n\
             class SystemUtils:\n    def print(self, message: str) -> None:\n        \
                 print(message)\n\n\n\
             @dataclass(frozen=True)\n\
             class Bye:\n    \"\"\"Says goodbye.\"\"\"\n\n    \
                 utils: Utils = field(default_factory=SystemUtils)\n\n    \
                 async def run(self, state: HasName) -> None:\n        \
                     self.utils.print(f\"Bye, {state.name}\")\n",
        );
        write(
            "tests/nodes/test_bye.py",
            "from dataclasses import dataclass, field\n\n\
             from demo_app.nodes.bye import Bye\n\n\n\
             @dataclass\nclass MockState:\n    name: str\n\n\n\
             @dataclass\nclass MockUtils:\n    \
                 printed: list[str] = field(default_factory=list[str])\n\n    \
                 def print(self, message: str) -> None:\n        \
                     self.printed.append(message)\n\n\n\
             async def test_says_goodbye() -> None:\n    \
                 utils = MockUtils()\n\n    \
                 await Bye(utils).run(MockState(name=\"Ann\"))\n\n    \
                 assert utils.printed == [\"Bye, Ann\"]\n",
        );
        write(
            "tests/walks/test_hello.py",
            "from dataclasses import dataclass, field\n\n\
             from demo_app.nodes.bye import Bye\n\
             from demo_app.nodes.greet import Greet\n\
             from demo_app.walks.hello import Hello, Id, State\n\n\n\
             @dataclass\nclass Script:\n    \
                 printed: list[str] = field(default_factory=list[str])\n\n    \
                 def print(self, message: str) -> None:\n        \
                     self.printed.append(message)\n\n\n\
             async def test_hello_greets_the_world_and_says_goodbye() -> None:\n    \
                 script = Script()\n    \
                 state = State()\n\n    \
                 await Hello(greet=Greet(script), bye=Bye(script)).run(Id.GREET, state)\n\n    \
                 assert state.name == \"world\"\n    \
                 assert script.printed == [\"Bye, world\"]\n",
        );
        assert!(ambler(&["check"], &demo), "the finished node and walk fail");

        std::fs::remove_dir_all(&folder).unwrap();
    }
}
