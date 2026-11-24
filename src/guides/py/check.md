# ambler check

    ambler check [--lint-only]

Run it from the project root when the work is done. It checks the structural conventions and prints one line per violation:

    <path>:<line>: <rule>: <what to do>

If there are none, it runs `uv run ruff format --check`, `uv run ruff check`, `uv run pyright` and `uv run pytest`, stopping at the first that fails. `--lint-only` skips those. It exits 1 if anything failed. Run `uv sync` first.

## Rules

- `layout`: `pyproject.toml` and `src/<package>/engine.py` exist, the package named after `[project] name`.
- `parse`: the file is valid Python.
- `todo`: no `not yet implemented` or `not yet wired` is left in a node, walk, utility or test.
- `file-name`: node, walk and utility modules are snake_case, not keywords.
- `node-shape`: a node module has its class, a `@dataclass(frozen=True)` with `async def run`; a `Utils` protocol; `SystemUtils`; and an `Outcome` enum of UPPER_SNAKE members, unless `run` returns `None`.
- `node-tests`: a node has `tests/nodes/test_<node>.py`.
- `orphan-test`: a test module's node or walk exists.
- `walk-surface`: a walk's `__all__` lists only `Error`, `main` and `run`, and it defines `main()` and `run()`.
- `walk-tests`: a walk has `tests/walks/test_<walk>.py`.
- `walk-runner`: a `[project.scripts]` entry runs the walk's `main`.
- `walk-spec`: the walk has `specs/<walk>.md`.
- `spec-nodes`: the spec's `###` headings are exactly the walk's `Id` members (`### Read Move` for `READ_MOVE`).
- `spec-sections`: a spec has `# Program Specifications`, `## Nodes` and `## Shared State`, and no other sections.
- `util-imports`: a utility imports nothing from the package's `engine`, `nodes` or `walks`.

What it cannot check is yours: that outcomes name what happened, that tests cover every branch, that the spec's prose matches the code. See `ambler help node-design`, `ambler help testing` and `ambler help spec`.
