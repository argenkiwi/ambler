# amble new

    amble new [<name>] [--example] --target ts

Creates the folder `<name>` with an empty TypeScript Ambler project. `<name>` is the package name: lowercase letters, digits, `-` and `_`. Fails if the folder exists. `--target ts` (or `AMBLE_TARGET=ts`) chooses TypeScript; without it the project is Rust.

Without `<name>`, it makes the current folder the project, named after the folder. The folder must be empty except for `.git`, and its name must be a valid package name.

It writes:

- `package.json` with the `test`, `check` and `lint` scripts and pinned Biome, TypeScript and Bun types; `tsconfig.json`; `biome.json`.
- `ambler.ts`, the engine, and empty `nodes/`, `walks/` and `utils/` folders.
- `README.md`, `.gitignore`.
- `AGENTS.md`, `CLAUDE.md` (which imports it) and `.agents/skills/ambler/SKILL.md`, with `.claude/skills` linked to `.agents/skills`. See `amble help agents`.

`--example` adds the counter walk as a reference: three nodes (`start`, `count`, `stop`) with their tests, the walk with its test, its spec and its `counter` script. Read it, then delete it when the project has its own walk.

Then run `bun install`. It does not run `git init`.
