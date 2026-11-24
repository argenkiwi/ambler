# ambler testing

Tests are async pytest functions: `tests/nodes/test_<node>.py` for nodes, `tests/walks/test_<walk>.py` for walks. `asyncio_mode = "auto"` runs them without decorators.

## Nodes: test first

`ambler node` writes a `run` that raises `not yet implemented` and one failing test per outcome, plus one for the error of a fallible node.

1. Finish the tests before the body. One test per branch: each outcome, the error, and each distinct way of reaching them.
2. Run `uv run pytest tests/nodes/test_<name>.py`. Every test must fail with `not yet implemented`. A pyright error means the contract is wrong: fix the types, not the tests. A pass means the body does not raise.
3. Implement `run` until they pass. Do not edit the tests in this step.

Rules:

- Name tests after the outcome and the condition: `test_reports_invalid_input_and_keeps_count`.
- `MockState` is a dataclass with the node's fields; `MockUtils` implements the `Utils` protocol by hand, no `unittest.mock`, so pyright checks it.
- Fake every util. Nothing in a node test touches the terminal, the clock, the network or the disk.
- Assert both the outcome and the state afterwards. For an error, `with pytest.raises(Error):` and then check the state was left alone.
- A helper such as `async def run(input, count) -> tuple[Outcome, int]` keeps one-line tests readable.

Fakes:

- Fixed behaviour: a field the method returns, `heads: bool`.
- Async util: `async def sleep(self, seconds: float) -> None: pass`.
- Failure: a field holding an exception the method raises.
- Recording what the node did: append to a list the test reads afterwards.

## Existing nodes without tests

Read the node, list its branches, and write the same kind of tests. They should pass at once; if one does not, the node has a bug or the test misreads it. Do not overwrite tests that already pass.

## Walks: test the wiring

`ambler walk` writes a `Script` and one test.

- Make `Script` take the inputs to play back (lists it pops from) and implement every node's `Utils`, recording what was printed or done.
- Build the walk with the script, `Counter(start=Start(script), …)`, and run it with `await walk.run(Id.START, state)` down one path.
- Assert the final state and what was recorded, in order.
- Cover every outcome at least once across the tests. Node logic is already tested in the nodes; here you prove each outcome leads where the spec says.
