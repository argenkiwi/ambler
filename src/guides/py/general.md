# amble

Scaffolds and checks Ambler projects: programs built as walks (state machines) of small, tested nodes. This project is Python (ambler-py, on uv); `amble` worked that out from `pyproject.toml` and the package's `engine.py`.

    amble new [<name>] [--example] [--target rust|ts|py]  create a project (no name: in this empty folder)
    amble node <name> …                  add a node: its contract, a raising `run`, failing tests
    amble util <name>                    add a module of side-effectful functions
    amble walk <name> --nodes … --edge … wire nodes into a walk, with its test, spec and script
    amble check [--lint-only]            verify the conventions, then ruff, pyright and pytest
    amble pave <spec> --agent '<cmd>'    implement a spec with an agent, one task per run
    amble help [topic]                   this guide, or one topic

Topics: a verb (new, node, util, walk, check, pave, help), or conventions, node-design, testing, spec, agents.

## The four ideas

- State: one dataclass per walk. Nodes ask for state through small protocols, `HasCount`, kept in `nodes/__init__.py`; the walk's `State` has every field they need, which pyright checks.
- Node: one step. A frozen dataclass whose `async def run(self, state)` mutates the state and returns a member of its `Outcome` enum: what happened, never which node comes next.
- Utils: a node's side effects, one method each, in its `Utils` protocol. `SystemUtils` is the real thing; tests pass their own.
- Walk: a `Walk[Id, State]` subclass whose `step` matches each node's outcomes to the next `Id`. The match has no `case _:`, so pyright fails on an unwired outcome.

## Layout

    src/<package>/engine.py     Node and Walk (identical in every project)
    src/<package>/walks/        one module per walk; each has a `main()` run by a pyproject.toml script
    src/<package>/nodes/        one module per node; the state protocols in __init__.py
    src/<package>/utils/        side-effectful functions nodes delegate to
    tests/nodes/  tests/walks/  one test module per node and per walk
    specs/                      one plain-language spec per walk

## Usual order of work

1. Describe the walk in `specs/<walk>.md` (`amble help spec`).
2. For each node: `amble node`, then tests, then the body (`amble help node-design`, `amble help testing`).
3. `amble walk` to wire them; finish the wiring test (`amble help walk`).
4. `amble check`.

Run `uv sync` once in a new project, so the checks and the formatter are there.
