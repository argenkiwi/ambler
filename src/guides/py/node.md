# amble node

    amble node <name> (--outcomes <a,b> | --terminal) [--fallible] [--state <field>[:<Type>]]...

Writes `src/<package>/nodes/<name>.py` and its tests, `tests/nodes/test_<name>.py`. Run it from the project root. Decide the flags with `amble help node-design` first: they are the node's contract.

- `<name>`: snake_case (or kebab-case), no `_node` suffix. The class is the same name in PascalCase.
- `--outcomes success,invalid_input`: one member of `Outcome` per thing that can happen, `SUCCESS` and `INVALID_INPUT`. The walk gives each one a successor.
- `--terminal`: the node ends the walk; `run` returns `None` and there is no `Outcome`. Give exactly one of `--outcomes` and `--terminal`.
- `--fallible`: adds `class Error(Exception)` for what stops the node, and a test that expects it.
- `--state count`: the node needs the existing `HasCount` protocol from `nodes/__init__.py`.
- `--state name:str`: declares a new `HasName` protocol there and requires it. Repeat `--state` for more; a node that needs several gets a local `State` protocol combining them.
- `--state board:my_app.board.Board`: a type that needs an import is given by its path. `nodes/__init__.py` imports it and the protocol uses `Board`; tests and walks that hold the field import it too.

What you get:

- `Outcome`, `Error` if fallible, an empty `Utils` protocol with its `SystemUtils`, and the node, a `@dataclass(frozen=True)` whose `utils` default to `SystemUtils()`.
- `async def run(self, state)` with the state protocol, raising `not yet implemented`.
- One test per outcome, and one for the error, with a `MockState` and a `MockUtils`. They fail with `not yet implemented`.

What is left to you, in this order:

1. Give `Utils` one method per side effect and implement each in `SystemUtils`.
2. Finish the tests: rename them, set up state and fakes, assert outcome and state (`amble help testing`).
3. Run `uv run pytest tests/nodes/test_<name>.py` and see them fail with `not yet implemented`.
4. Implement `run`. Do not change the tests while doing so.
5. Replace the `TODO` docstrings.

It never overwrites: it fails if the node exists.
