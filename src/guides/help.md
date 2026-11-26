# ambler help

    ambler help [topic]

Prints a guide on standard output. Without a topic it prints the overview: the verbs, the four ideas, the layout and the usual order of work.

The guides follow the project's target: run inside a TypeScript project (or with `--target ts`, or `AMBLER_TARGET=ts`) they describe ambler-ts; inside a Python project (or with `--target py`) ambler-py; otherwise ambler-rs. Every verb works out the target the same way, so the flag is only needed for `ambler new`.

Topics:

- `new`, `node`, `util`, `walk`, `check`, `pave`, `help`: what each verb takes, writes and leaves to you.
- `conventions`: every convention in one page, with the reason for each.
- `node-design`: how to choose a node's outcomes, errors, state and utils.
- `testing`: test-first nodes, mocks, and wiring tests for walks.
- `spec`: the format of `specs/<walk>.md`.
- `agents`: how `AGENTS.md` and the skill relate to this command, and the text to add to a project that lacks them.

`node`, `util`, `walk`, `check`, `conventions`, `node-design`, `testing` and `spec` are printed by the project's own CLI (`cargo ambler help <topic>`, `bun run ambler help <topic>` or `uv run ambler help <topic>`), so they match the version the project pins and can be read where `ambler` is not installed, such as in a container. They are handed over as `node` or `check` are.

An unknown topic lists the topics on standard error and exits 1. So does an unknown verb.
