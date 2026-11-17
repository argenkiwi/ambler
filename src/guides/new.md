# amble new

    amble new <name> [--example]

Creates the folder `<name>` with an empty Ambler project. `<name>` is the Cargo package name: lowercase letters, digits, `-` and `_`. Fails if the folder exists.

It writes:

- `Cargo.toml` with `thiserror`, `tokio` and the lint settings (`unsafe_code` forbidden, clippy warnings on).
- `src/lib.rs`, `src/engine.rs` and empty `src/nodes/`, `src/walks/` and `src/utils/` modules.
- `README.md`, `.gitignore`.
- `AGENTS.md`, `CLAUDE.md` (which imports it) and `.agents/skills/ambler/SKILL.md`, with `.claude/skills` linked to `.agents/skills`. See `amble help agents`.

`--example` adds the counter walk as a reference: three nodes (`start`, `count`, `stop`), a utility module, the walk, its spec and `src/main.rs`. Read it, then delete it when the project has its own walk.

Without `--example` there is no `src/main.rs` yet: the first `amble walk` writes it.

It does not run `git init`.
