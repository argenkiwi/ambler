# Program Specifications

`amble check [--lint-only]` verifies that the project in the current directory follows the Ambler conventions of its target, then that it is formatted, free of lint warnings and passes its tests.

The walk starts at `INSPECT`. If an argument other than `--lint-only` (or its old name, `--no-cargo`) is given, it starts at `REPORT` with the usage as the reason.

For a Python or TypeScript project, `amble check` first hands the check to the project's own runner (`uv run ambler-cli check`, `bun run ambler check`) when `.venv/bin/ambler-cli` or `cli.ts` exists and `uv`/`bun` is installed. Otherwise it prints how to install it (`uv sync`, `bun install`) and runs the target's checks itself, as below. It never downloads the runner for `check`. `amble node`, `walk` and `util` try `uvx`/`bunx` before falling back to amble's own scaffolding. See `docs/tooling_protocol.md`.

## Nodes

### Inspect
- Reads the project's manifest, sources and specs and checks them against the structural conventions, printing one line per violation with its file, line and rule.
- If there are no violations, it transitions to `VERIFY`, or terminates if `--lint-only` was given.
- If there are violations, it records how many and transitions to `REPORT`.
- If the project cannot be read the program fails with that error.

### Verify
- Runs the target's checks in order, printing each command before running it: for Rust the formatting check, clippy with warnings denied and the tests; for TypeScript the type check, Biome's lint and format check, and the tests; for Python ruff's format check, ruff's lint, pyright and the tests.
- If all pass, it terminates.
- If one fails, it skips the rest, records which and transitions to `REPORT`.
- If a check command cannot be run the program fails with that error.

### Report
- Prints the recorded reason on standard error and terminates. The program exits 1.

## Shared State

The shared state consists of:
- `target`: the language of the project.
- `message`: why the check failed, if it did.
