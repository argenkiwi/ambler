# ambler testing

Tests live inline, in a `#[cfg(test)] mod tests` at the bottom of the file they test. There is no `tests/` folder.

## Nodes: test first

`ambler node` writes a body of `todo!()` and one failing test per outcome and per error.

1. Finish the tests before the body. One test per branch: each outcome, each error, and each distinct way of reaching them.
2. Run `cargo test nodes::<name>::`. Every test must fail with `not yet implemented`. A compile error means the contract is wrong: fix the types, not the tests. A pass means the body is not `todo!()`.
3. Implement `run` until they pass. Do not edit the tests in this step.

Rules:

- Name tests `<expected_behaviour>_<condition>`: `reports_invalid_input_and_keeps_count`.
- `MockState` implements only the traits the node requires.
- Mock every util. Nothing in a node test touches the terminal, the clock, the network or the disk.
- Assert the outcome and the state. Never assert which node comes next: the node does not know.
- Use `#[tokio::test]` and `.await`.
- Infallible node: `let Ok(outcome) = Node::with_utils(mock).run(&mut state).await;`. Fallible: `.unwrap()`, or `assert!(matches!(result, Err(Error::Io(_))))` for the failure, and assert the state was left alone.

Mocks:

- Fixed behaviour: fields on the mock, `MockUtils { heads: true }`.
- Async util: `async fn sleep(&self, _duration: Duration) {}`.
- Recording what the node did: keep a `Mutex` in the mock, implement `Utils for &MockUtils`, pass `Node::with_utils(&mock)` and read the mock afterwards.

## Existing nodes without tests

Read the node, list its branches, and write the same kind of tests. They should pass at once; if one does not, the node has a bug or the test misreads it. Do not overwrite tests that already pass.

## Walks: test the wiring

`ambler walk` writes a `Script` and one test. `Script` implements every node's `Utils` for `&Script`.

- Give `Script` a `Mutex<VecDeque<_>>` per input to play back and a `Mutex<Vec<_>>` per effect to record.
- Run the walk from its start id with a script that drives it down one path.
- Assert the final state and what was recorded, in order.
- Cover every edge at least once across the tests. Node logic is already tested in the nodes; here you prove each outcome leads where the spec says.
