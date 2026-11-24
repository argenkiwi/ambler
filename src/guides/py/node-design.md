# amble node-design

How to decide what a node is before running `amble node`. Its outcomes, error, state protocols and utils are the node's contract with its tests and with every walk that uses it. If they turn out wrong, change them and the tests first, then the body.

## Reuse first

Look in `specs/` and `src/<package>/nodes/` for a node that already does this. A node is reusable across walks because it asks only for the protocols it needs. If a walk names a field differently, rename it in the walk; do not write an adapter node.

## One step

A node does one thing a spec can describe in a sentence or two. If the description needs "and then", it is probably two nodes with an outcome between them.

## Outcomes

- One member per distinct thing that can happen and that a walk might route differently: `SUCCESS` and `INVALID_INPUT`, `AGAIN` and `FINISHED`.
- Name what happened, not where to go. `FINISHED`, not `GO_TO_STOP`. The node never names its successor.
- A node that ends the walk is terminal: `run` returns `None`.
- A problem the program handles as part of its flow (bad input, a missing file it can create) is an outcome. A problem that stops the walk raises the node's `Error`.
- A node that reads input routes end of input (`None`) somewhere other than its own retry outcome, or it loops forever.

## Errors

- `--fallible` gives the node `class Error(Exception)`. Raise it with a message that says what could not be done, chained to the cause: `raise Error("could not read the move") from e`.
- Never swallow an exception with a bare `except`. Catch only what a util documents it raises.

## State

- Ask only for the fields the node reads or writes, through `Has*` protocols: `--state count` reuses `HasCount`, `--state name:str` declares `HasName`.
- Mutate the state in place: `state.count += 1`.

## Utils

- Every side effect goes through `self.utils`: printing, reading input, time, randomness, files, network, processes. `run` stays deterministic given its utils.
- One method per effect, in the terms of the domain: `async def sleep(self, seconds: float) -> None`, `def read_line(self, message: str) -> str | None`.
- A failure the node must handle is in the return type; an unexpected one raises (`OSError` and the like), and the method's docstring says so.
- `SystemUtils` implements each method in a line or two. If that needs a package, is shared with another node or is long, it belongs in `utils/` (`amble help util`).
- A node with no side effects keeps its `Utils` protocol empty.

## The body

- Return `Outcome.X` (or `None` when terminal) and nothing else: no ids, nothing from `walks/`.
- `run` is always `async`; await the utils that are.
