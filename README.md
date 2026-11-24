# ambler

The common entry point of a command line for coding agents (and people) that scaffolds and checks [Ambler](https://github.com/argenkiwi/ambler-rs) projects: programs built as walks (state machines) of small, tested nodes. Projects are Rust ([ambler-rs](https://github.com/argenkiwi/ambler-rs)) by default, TypeScript on Bun ([ambler-ts](https://github.com/argenkiwi/ambler-ts)) with `--target ts`, or Python on uv ([ambler-py](https://github.com/argenkiwi/ambler-py)) with `--target py`.

Ambler is a template, not a library, so every project carries its own copy of the engine and has to follow the same conventions by hand. `ambler` itself only does what applies to every target (`new`, `pave`, `help`, working out the target) and hands `node`, `walk`, `util` and `check` to the native CLI of each language, which does the mechanical part (files in the right place, registration, wiring boilerplate, convention checks). The design is left to whoever is building: which outcomes a node has, what its tests prove, where each outcome leads.

## Install

```sh
cargo install --git https://github.com/argenkiwi/ambler
```

## Use

```sh
ambler new my-app --example        # a project, with the counter walk as a reference
cd my-app                         # (or run `ambler new` in an empty folder to use it in place)

ambler node ask --outcomes answered,invalid --fallible --state name:String
ambler node greet --terminal --state name
#   … write each node's tests, see them fail, implement run …

ambler walk hello --nodes ask,greet --edge ask.Answered=greet --edge ask.Invalid=ask
ambler util terminal               # a module for side effects nodes share
ambler check                       # conventions, then fmt, clippy and tests
ambler pave hello --agent 'claude -p'  # or: let an agent implement specs/hello.md, one task per run
ambler help [topic]                # the conventions, one guide per topic
```

| Verb | Does |
|---|---|
| `new <name> [--example]` | Creates the project: engine, module folders, lint settings, agent context files. |
| `node <name> …` | Writes a node's contract (`Outcome`, `Error`, `Utils`, state bounds), a `todo!()` body and one failing test per branch; registers it; declares new `Has*` state traits. |
| `util <name>` | Writes and registers a utility module. |
| `walk <name> --nodes … [--edge …]` | Reads the nodes and writes the walk: `State`, `Id`, the exhaustive `step`, `run()`, a wiring-test skeleton, the spec and the runner. Edges not given are left as `todo!()` arms. |
| `check [--lint-only]` | Reports each broken convention as `path:line: rule: message`, then runs `cargo fmt --check`, clippy with warnings denied and the tests. |
| `pave <spec> --agent '<cmd>' …` | Implements a spec by running a coding agent in a loop, one narrow task per run (create or finish a node, wire the walk, repair the check), committing a checkpoint after every passing check. Rolls back and stops if the agent gets stuck. |
| `help [topic]` | Prints a guide: a verb, or `conventions`, `node-design`, `testing`, `spec`, `agents`. |

Nothing is ever overwritten: a verb fails if a file it would create exists.

## Targets

`ambler new my-app --target ts` (or `AMBLER_TARGET=ts ambler new my-app`) creates a TypeScript project: `ambler.ts`, `package.json` with Biome and `tsc`, and `nodes/`, `walks/` and `utils/` with tests beside them in `tests/` folders. Run `bun install` in it once.

`ambler new my-app --target py` creates a Python project: `pyproject.toml` with strict pyright, ruff and pytest, and a `src/my_app/` package holding `engine.py` and the `nodes`, `walks` and `utils` packages, with tests in `tests/nodes/` and `tests/walks/`. Each walk runs as a `[project.scripts]` entry, `uv run <walk>`. Run `uv sync` in it once.

Inside a project the flag is not needed: every verb works out the target from `--target`, then `AMBLER_TARGET`, then the project's files (`ambler.ts` and `package.json` for TypeScript, `Cargo.toml` and `src/engine.rs` for Rust, `pyproject.toml` and `src/<package>/engine.py` for Python), and is Rust if none is found. The verbs and flags are the same; `ambler help` describes the conventions of the project's target.

### Native CLIs

`ambler node`, `walk`, `util` and `check` run the project's own CLI, as defined by the [Ambler Tooling Protocol](docs/tooling_protocol.md), and end with its exit code. ambler has no fallback of its own: if the CLI is missing it says how to install it and exits 2.

| Target | CLI | Install |
|---|---|---|
| Rust | `cargo ambler` | `cargo install --git https://github.com/argenkiwi/ambler-rs cargo-ambler` |
| TypeScript | `bun run ambler` | `bun install` in the project (`bunx` fetches it for `node`, `walk` and `util`) |
| Python | `uv run ambler-cli` | `uv sync` in the project (`uvx` fetches it for `node`, `walk` and `util`) |

## Agents

The conventions are written down in one place, the guides compiled into the binary, next to the code that enforces them. The context files only point there:

- `AGENTS.md`: a few standing rules, always in context (create with `ambler`, test first, finish with `ambler check`). `CLAUDE.md` imports it.
- `.agents/skills/ambler/SKILL.md`: one skill that triggers on requests like "add a step" and routes each to a verb and a guide. Install it globally with `npx skills add argenkiwi/ambler` so that an agent in an empty folder knows to start with `ambler new`.

`ambler new` writes both into every project. They are this repository's own files: ambler is an Ambler project, follows the same instructions and passes its own `ambler check`.

## Layout

```
src/engine.rs    Node and Walk traits, Transition; also what `ambler new` writes as the engine
src/main.rs      Runs the walk for the verb
src/walks/       One walk per verb: `delegate` (node, walk, util, check), `new`, `pave`, `help`
src/nodes/       delegate runs a native CLI; plan_project decides what `new` writes; write, report, guide and the `pave` nodes act
src/scaffold/    Pure text in, text out: new-project files, the source reader `pave` uses
src/utils/       File system and process access
src/guides/      The guides `ambler help` prints
templates/       Static files for new projects, and the counter example
specs/           One spec per walk
```

## Development

```sh
cargo run -- check                    # ambler checks itself
cargo test -- --ignored               # also generate a project and check it end to end
```
