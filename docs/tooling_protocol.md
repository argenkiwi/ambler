# Ambler Tooling Protocol (ATP) Specification

The **Ambler Tooling Protocol (ATP)** defines the standardized CLI contract that each language-specific Ambler implementation (`ambler-py`, `ambler-ts`, `ambler-rs`, `ambler-kt`, `ambler-mojo`, etc.) implements.

By adhering to this protocol, each repository is self-contained (developers can work natively without installing foreign toolchains), while the unified orchestrator `ambler` can transparently discover and delegate tasks across all supported languages.

`ambler` has no fallback of its own: it is the common entry point for what applies to every target (`new`, `pave`, `help`) and hands `node`, `walk`, `util` and `check` to the native CLI. If there is none it exits 2 with how to install it; otherwise it ends with the CLI's exit code. `new` is not part of the protocol: it runs before a project, and so its CLI, exists.

---

## 1. Tooling Discovery and Invocation

Each target repository exposes a command-line runner through its standard package manager:

| Target | Native Tool | Invocation by `ambler` |
|---|---|---|
| **Rust** (`ambler-rs`) | `ambler` crate: `ambler::cli::run`, run by the project's `xtask` behind a `cargo ambler` alias (or the `cargo-ambler` binary) | `cargo ambler <command>` |
| **TypeScript** (`ambler-ts`) | `ambler` package: a dev dependency whose `ambler` bin lives in `node_modules/.bin` (not a PATH binary) | `bun run ambler <command>` |
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
* Registers or updates shared state definitions if required by the target conventions.

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

**Exit Codes:**
* `0`: All checks, formatting, and tests passed.
* `1`: Check failure (lint violation, type error, or test failure).
* `2`: Invalid arguments or unexpected execution error.
