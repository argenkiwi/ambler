# ambler agents

How coding agents learn to work in an Ambler project. Three places, each with one job, and none of them restates the conventions:

- The project's own CLI holds the reference. `cargo ambler help <topic>` (Rust), `bun run ambler help <topic>` (TypeScript) or `uv run ambler help <topic>` (Python) is the only place a convention is written down. It ships with the CLI that enforces it, so the two cannot drift apart, and it is a dev dependency of the project, so an agent can read it where `ambler` is not installed, such as in a container.
- `AGENTS.md` holds behaviour: a few standing rules that are always in the agent's context, each pointing at a guide. `CLAUDE.md` is one line that imports it.
- `.agents/skills/ambler/SKILL.md` is a router: it triggers on requests such as "add a step" and maps each to a verb and a guide of the project's CLI. ambler's own skill, installed globally with `npx skills add argenkiwi/ambler`, tells an agent in an empty folder to start with `ambler new`.

`ambler new` writes all three, naming the project's CLI, and links `.claude/skills` to `.agents/skills`.

For a project that was not created by `ambler new`, or one created before its context files named its own CLI, use the text below as its `AGENTS.md`, with `<cli>` replaced by the project's CLI, `<engine>` by its engine file and `<setup>` by its install command (`cargo build`, `bun install` or `uv sync`). Copy `templates/SKILL.md` from ambler's repository, filled in the same way, if you want the triggers. There is no verb for this: it is done once per project.

---

