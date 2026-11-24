# amble new

    amble new [<name>] [--example] --target py

Creates the folder `<name>` with an empty Python Ambler project. `<name>` is the project name: lowercase letters, digits, `-` and `_`, starting with a letter. The package is the name in snake_case: `my-app` is imported as `my_app`. Fails if the folder exists. `--target py` (or `AMBLE_TARGET=py`) chooses Python; without it the project is Rust.

Without `<name>`, it makes the current folder the project, named after the folder. The folder must be empty except for `.git`, and its name must be a valid project name.

It writes:

- `pyproject.toml`: uv_build, strict pyright with `reportMatchNotExhaustive`, ruff's rules, pytest with asyncio, and pinned dev tools; an empty `[project.scripts]` for the walks.
- `src/<package>/engine.py`, the engine, and the `nodes`, `walks` and `utils` packages.
- `README.md`, `.gitignore`.
- `AGENTS.md`, `CLAUDE.md` (which imports it) and `.agents/skills/ambler/SKILL.md`, with `.claude/skills` linked to `.agents/skills`. See `amble help agents`.

`--example` adds the counter walk as a reference: three nodes (`start`, `count`, `stop`), the `HasCount` protocol, a terminal utility, their tests, the walk with its test, its spec and its `counter` script. Read it, then delete it when the project has its own walk.

Then run `uv sync`. It does not run `git init`.
