# ambler conventions

Every convention of a Python Ambler project, with why it is there. `ambler check` enforces the structural ones.

## Nodes

- A node is a `@dataclass(frozen=True)` named after its module, with `utils: Utils = field(default_factory=SystemUtils)` and `async def run(self, state)`. It satisfies the engine's `Node` protocol structurally; it does not subclass it.
- `run` returns a member of the module's `Outcome` enum, named for what happened: `SUCCESS`, `INVALID_INPUT`. It never names the next node: wiring is the walk's job, so a node can be reused in any walk. A terminal node returns `None` and has no `Outcome`.
- A node that cannot finish raises its module's `class Error(Exception)`, chaining the cause with `raise Error(...) from e`. An expected problem is an outcome. A node that cannot fail defines no `Error`.
- `Utils` is a `Protocol`, one method per side effect; `SystemUtils` implements it. `Count()` runs for real and `Count(fake)` is for tests.
- State is mutated in place. A node asks for it through `Has*` protocols in `nodes/__init__.py`, one field each, and requires only the fields it touches; several are combined in a node-local `State` protocol.

## Walks

- A walk is a frozen dataclass subclassing `Walk[Id, State]`, with one field per node defaulting to the real node. Tests build it with fakes: `Counter(start=Start(script), …)`.
- `step` matches the `Id`, then each node's outcome, with no `case _:`. pyright's strict mode and `reportMatchNotExhaustive` make a forgotten node or outcome a type error; without them a missing case silently halts the walk.
- A walk module's `__all__` is `Error` (if any), `main` and `run`. `State`, `Id` and the walk class keep public names because pyright's strict mode rejects a test importing `_private` names.
- `main()` runs the walk and turns its `Error` into an exit message; a `[project.scripts]` entry names it.

## Tests

- `tests/nodes/test_<node>.py` and `tests/walks/test_<walk>.py`, async pytest functions (`asyncio_mode = "auto"`).
- Fakes are small hand-written classes, not `unittest.mock`, so pyright checks them against the `Utils` protocol.
- Nodes are built test-first.

## Files

- `src/<package>/nodes/<name>.py`, `src/<package>/walks/<name>.py`, `src/<package>/utils/<name>.py`: snake_case. Nothing is registered.
- `specs/<walk>.md` for every walk.
- `src/<package>/engine.py` holds `Node` and `Walk`. It is copied in, not a dependency, and is not edited.

## Project

- pyright runs in strict mode and ruff's `E`, `F`, `I`, `UP`, `B`, `SIM` and `RUF` rules are errors in `ambler check`, as is `ruff format`.
- Tests import with pytest's `importlib` mode, so a node and a walk may share a name.
