# Ambler Tooling Protocol (ATP) Specification

The **Ambler Tooling Protocol (ATP)** defines the standardized CLI contract that each language-specific Ambler implementation (`ambler-py`, `ambler-ts`, `ambler-rs`, `ambler-kt`, `ambler-mojo`, etc.) implements.

By adhering to this protocol, each repository is self-contained (developers can work natively without installing foreign toolchains), while the unified orchestrator `amble` can transparently discover and delegate tasks across all supported languages.

`amble` has no fallback of its own: it is the common entry point for what applies to every target (`new`, `pave`, `help`) and hands `node`, `walk`, `util` and `check` to the native CLI. If there is none it exits 2 with how to install it; otherwise it ends with the CLI's exit code. `new` is not part of the protocol: it runs before a project, and so its CLI, exists.

---

## 1. Tooling Discovery and Invocation

Each target repository exposes a command-line runner through its standard package manager:

| Target | Native Tool | Invocation by `amble` |
|---|---|---|
| **Rust** (`ambler-rs`) | `cargo ambler` (the `cargo-ambler` crate) | `cargo ambler <command>` |
| **TypeScript** (`ambler-ts`) | Bun script or `ambler` CLI | `bun run ambler <command>` |
| **Python** (`ambler-py`) | `ambler-cli` (or `python -m ambler.cli`) | `uv run ambler-cli <command>` |
| **Kotlin** (`ambler-kt`) | Kotlin wrapper script / Gradle task | `./kotlin ambler <command>` |
| **Mojo** (`ambler-mojo`) | Pixi task | `pixi run ambler <command>` |

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
