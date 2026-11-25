# Program Specifications

`ambler node`, `ambler walk`, `ambler util` and `ambler check` hand the verb, with its arguments, to the native Ambler CLI of the project's target, and end with the exit code it ends with. ambler does none of this work itself: each target's CLI scaffolds and checks its own language. See `docs/tooling_protocol.md`.

## Nodes

### Delegate
- Initial node.
- Works out where to run the verb: the project's own CLI (`cargo ambler` for Rust, `bun node_modules/.bin/ambler` for TypeScript, since `bun run ambler` would run the project's own `ambler.ts` unless the project has an `ambler` script, `uv run ambler` for Python) when it is installed. `ambler new` makes each one a dev dependency of the project, run as a project script, so nothing has to be on the `PATH`; Rust uses an `xtask` crate behind a `cargo ambler` alias; any `ambler` alias under `[alias]` in `.cargo/config.toml` (or `.cargo/config`) counts on its own, even without an `xtask` crate (ambler-rs itself runs its own binary that way), and a global `cargo-ambler` also works. Projects made before that still run through their own `cli.ts` and `ambler-cli`. For `node`, `walk` and `util` in a TypeScript or Python project that has not installed it, it prints how to get it (install what the project already depends on, or add the dependency, with the exact command) and runs the CLI with `bunx` or `uvx` instead, if that is installed. `check` never downloads the CLI.
- Runs the CLI with the verb and its arguments unchanged, sharing the terminal, and keeps its exit code.
- If the CLI ran, it terminates, whatever its exit code.
- If there is no CLI to run, or it could not be started, it records how to install it, sets the exit code to 2 and transitions to `REPORT`.

### Report
- Prints the recorded reason on standard error and terminates.

## Shared State

The shared state consists of:
- `target`: the language of the project.
- `verb`: the verb handed to the CLI.
- `args`: the arguments after the verb.
- `exit_code`: the exit code ambler ends with.
- `message`: why the verb could not be run, if it could not.
