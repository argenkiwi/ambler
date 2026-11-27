# Ambler Tooling Protocol (ATP) Specification

The **Ambler Tooling Protocol (ATP)** defines the standardized CLI contract that each language-specific Ambler implementation (`ambler-py`, `ambler-ts`, `ambler-rs`, `ambler-kt`, `ambler-mojo`, etc.) implements.

By adhering to this protocol, each repository is self-contained (developers can work natively without installing foreign toolchains), while the unified orchestrator `ambler` can transparently discover and delegate tasks across all supported languages.

`ambler` has no fallback of its own: it is the common entry point for what applies to every target (`new`, `pave`, `help`) and hands `node`, `walk`, `util`, `check` and the component and convention guides of `help` to the native CLI. If there is none it exits 2 with how to install it; otherwise it ends with the CLI's exit code. `new` is not part of the protocol: it runs before a project, and so its CLI, exists.

---

## 1. Tooling Discovery and Invocation

Each target repository exposes a command-line runner through its standard package manager:

| Target | Native Tool | Invocation by `ambler` |
|---|---|---|
| **Rust** (`ambler-rs`) | `ambler` crate: `ambler::cli::run`, run by the project's `xtask` behind a `cargo ambler` alias (or any `ambler` alias in `.cargo/config.toml`, or the `cargo-ambler` binary) | `cargo ambler <command>` |
| **TypeScript** (`ambler-ts`) | `ambler` package: a dev dependency whose `ambler` bin lives in `node_modules/.bin` (not a PATH binary) | `bun node_modules/.bin/ambler <command>` (never `bun run ambler` without an `ambler` script: `bun run` prefers the project's own `ambler.ts` to the bin, runs the library and exits 0) |
| **Python** (`ambler-py`) | `ambler` package: a dev dependency whose `ambler` script lives in `.venv/bin` (not a PATH binary); `ambler-cli` is a deprecated alias | `uv run ambler <command>` |
| **Kotlin** (`ambler-kt`) | Kotlin wrapper script / Gradle task | `./kotlin ambler <command>` |
| **Mojo** (`ambler-mojo`) | Pixi task | `pixi run ambler <command>` |

### Naming

`ambler` is the orchestrator's binary and owns that name on `PATH`. A native CLI must never install an executable called `ambler` **globally**: it is installed into the project as a dev dependency and run as a project-local script, task or wrapper (`bun run ambler`, `uv run ambler`, `cargo ambler` via an `xtask` alias, `pixi run ambler`, `./kotlin ambler`), so nothing has to be on the `PATH`. A package, crate or script may be called `ambler` because it never lands on the `PATH`; `ambler new` adds the dependency from GitHub to every project it creates, and `ambler` says how to install it when it is missing. A project's own program binaries are named after what they run (`counter`, `ambler-counter`), never `ambler`.

---

## 2. Standard Verbs & Signatures

### 2.1 `node`
Scaffolds a new node module and its corresponding unit tests.

```sh
<tool> node <name> (--outcomes <a,b> | --terminal) [--fallible] [--state <field>[:<Type>]]...
```

* `<name>`: Snake_case or kebab-case name of the node.
* `--outcomes <a,b>`: Comma-separated list of outcome branches.
* `--terminal`: Indicates that the node terminates the walk (no outgoing edges / `onDone`).
* `--fallible`: Indicates that the node's `run` function can raise an error / exception.
* `--state <field>[:<Type>]`: State fields read or mutated by the node.

**Behavior:**
* Creates the node file in the language's nodes directory (`nodes/<name>.ts`, `src/<pkg>/nodes/<name>.py`, `src/nodes/<name>.rs`).
* Creates failing unit test skeletons in the tests directory (`nodes/tests/<name>.test.ts`, `tests/nodes/test_<name>.py`).
* Each `--state <field>:<Type>` adds a `Has<Field>` contract to the state registry (§3) if none exists; a type that differs from the existing contract's is an error (exit 2). A bare `--state <field>` reuses the existing contract, and is an error when there is none.
* The node composes the contracts it was given, and nothing else.

### 2.2 `walk`
Wires existing nodes into a state machine walk.

```sh
<tool> walk <name> --nodes <start,next,...> [--edge <node>.<Outcome>=<target>]... [--no-runner]
```

* `<name>`: Name of the walk.
* `--nodes <list>`: Comma-separated list of nodes in the walk; the first is the start node.
* `--edge <node.Outcome=target>`: Transition mappings from node outcomes to target nodes or halt.
* `--no-runner`: Suppresses creating an executable entry point.

**Behavior:**
* Creates the walk definition file (`walks/<name>.ts`, `src/<pkg>/walks/<name>.py`, `src/walks/<name>.rs`).
* Creates the wiring integration test (`walks/tests/<name>.test.ts`, `tests/walks/test_<name>.py`).
* Creates the specification template (`specs/<name>.md`).
* Builds the walk's `State` from the contracts its nodes compose, read from the state registry (§3); an unknown contract or a field two contracts type differently is an error (exit 2).
* Adds a runner entry point (e.g. in `pyproject.toml` scripts, `package.json` scripts, or `main.rs`) unless `--no-runner` is specified.

### 2.3 `util`
Adds a shared utility helper module.

```sh
<tool> util <name>
```

* `<name>`: Name of the utility module.

**Behavior:**
* Creates the utility interface and production implementation in `utils/`.

### 2.4 `check`
Verifies conventions, formatting, types/lints, and tests.

```sh
<tool> check [--lint-only]
```

* `--lint-only`: Runs convention checks and linters without executing the test suite.

The convention checks include the state contract rules of §3 (`state-traits` in Rust, `state-protocols` in Python, `state-contracts` in TypeScript); a violation names the rule and exits `1`.

**Exit Codes:**
* `0`: All checks, formatting, and tests passed.
* `1`: Check failure (lint violation, type error, or test failure).
* `2`: Invalid arguments or unexpected execution error.

### 2.5 `help`
Prints the guides to a project's components and conventions.

```sh
<tool> help [topic]
```

* `[topic]`: one of `node`, `util`, `walk`, `check`, `conventions`, `node-design`, `testing`, `spec`. Without one, the general guide: the verbs, the topics, the exit codes, the layout and the usual order of work.

**Behavior:**
* Prints the guide on standard output and exits `0`. The guides are embedded in the CLI (compiled in, a text import or package data), never fetched, so a project reads the guides of the version it pins, offline.
* An unknown topic lists the topics on standard error and exits `1`.
* Guides name the native invocation (`cargo ambler node`, `bun run ambler node`, `uv run ambler node`), never the global `ambler`, so an agent can follow them where only the project's dependencies are installed, such as in a container.
* `--help` and `-h` print the usage.

`ambler help <topic>` hands these topics to the native CLI. `ambler` keeps the guides that apply before or outside a project: the overview, `new`, `pave`, `help` and `agents`.

---

## 3. State Contracts

A node declares the part of the walk's state it needs through *state contracts*, so it works with any walk whose state has those fields. Every target follows the same rules:

1. **One registry.** All contracts of a project live in one module, the *state registry*. No other file declares one.
2. **One field each.** A contract is named `Has<Field>` (PascalCase of the field) and covers exactly one field.
3. **Nodes compose, never declare.** A node requires only the contracts for the fields it reads or writes, combined in the target's own way. It declares no state fields of its own.
4. **The walk's `State` satisfies them.** It has every field its nodes' contracts name, and no field that no node uses. The compiler or type checker verifies this.
5. **Tools read the registry.** `node --state` adds or reuses contracts there; `walk` builds `State` from it; `check` enforces rules 1–3.

| Target | Registry | Contract | A node composes | No state |
|---|---|---|---|---|
| **Rust** | `src/nodes/mod.rs` | `trait HasCount { fn count(&self) -> u32; fn count_mut(&mut self) -> &mut u32; }` (`_mut` optional: read-only contracts are allowed; no other methods) | `where S: HasCount + HasExitCode + Send` | `where S: Send` |
| **TypeScript** | `nodes/state.ts` | `export interface HasCount { count: number }` | `export interface State extends HasCount, HasExitCode {}` | `export type State = object;` |
| **Python** | `src/<pkg>/nodes/__init__.py` | `class HasCount(Protocol): count: int` | `run(self, state: HasCount)`, or a local `class State(HasCount, HasExitCode, Protocol): pass` | `state: object` |

A contract's field may have any type, a struct or record included; what it may not do is name a second field, or offer behaviour beyond reading and writing its one field. Behaviour belongs in the node, or in its utils when it is a side effect.
