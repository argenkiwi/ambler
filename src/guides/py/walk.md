# ambler walk

    ambler walk <name> --nodes <start,next,...> [--edge <node>.<Outcome>=<node|halt>]... [--no-runner]

Wires existing nodes into `src/<package>/walks/<name>.py`. Run it from the project root, after the nodes exist: it reads their outcomes, errors, utils and state from `nodes/`. If a node is missing it lists the `ambler node` commands to run first.

- `--nodes start,count,stop`: the nodes of the walk. The first is where it starts.
- `--edge start.success=count`: where an outcome leads, by its name in any case (`success`, `SUCCESS`, `Success`). Repeat it, or separate edges with commas. `=halt` returns `None`, which ends the walk.
- A terminal node (`run` returns `None`) ends the walk.
- Outcomes you leave out become `raise NotImplementedError("not yet wired: NODE.OUTCOME")`: the walk type-checks, and raises when it reaches that outcome until you wire it.
- `--no-runner`: do not add a `[project.scripts]` entry.

What it writes:

- `src/<package>/walks/<name>.py`:
  - `State`, a dataclass with every field of every node's protocols, each with a default. It fails if two nodes give a field different types.
  - `Id`, one member per node in capitals: `READ_MOVE`.
  - The walk class, a frozen dataclass of its nodes, whose `step` matches every outcome of every node.
  - `Error`: a node's `Error`, or a tuple of them if several nodes can fail; none if none can.
  - `run()`, which walks from the first node and a fresh `State`, and `main()`, which runs it and exits with the error.
- `tests/walks/test_<name>.py`, with a `Script` to fill in and one test.
- `specs/<name>.md`, if there is none, with a section per node (`ambler help spec`).
- A `<name>` script in `pyproject.toml`: run it with `uv run <name>`.

## Rules for a walk

- `__all__` lists only `Error` (if there is one), `main` and `run`. `State`, `Id` and the walk class stay importable for the wiring test.
- Every outcome has a `case`. Never add `case _:`: pyright's `reportMatchNotExhaustive` is what makes a new outcome fail until it is wired.
- Run it with the engine's `Walk.run`. Do not write the loop yourself.
- The initial state is built in `run()`. If the program takes arguments, read `sys.argv` there.
- State the nodes do not share does not belong in `State`.

## After generating

1. Wire the remaining `not yet wired` arms.
2. Finish `Script` and the wiring test, one test per path (`ambler help testing`).
3. Fill in the spec.
4. `uv sync` (so the new script is installed), run it, then `ambler check`.

Changing a walk later (adding a node, moving an edge) is an edit by hand: the import, the `Id` member, the field and the `case`; pyright points at each one you miss. Update the spec in the same change.
