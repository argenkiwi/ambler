# amble node-design

How to decide what a node is before running `amble node`. The outcome, error, state and utils types are the node's contract with its tests and with every walk that uses it. If they turn out wrong, change them and the tests first, then the body.

## Reuse first

Look in `specs/` and `src/nodes/` for a node that already does this. A node is reusable across walks because it only knows the state through traits. If a walk names its fields differently, implement the trait on that walk's `State`; do not write an adapter node.

## One step

A node does one thing a spec can describe in a sentence or two. If the description needs "and then", it is probably two nodes with an edge between them.

## Outcomes

- One variant per distinct thing that can happen and that a walk might route differently: `Success` and `InvalidInput`, `Again` and `Finished`.
- Name what happened, not where to go. `Finished`, not `GoToStop`. The node never names its successor.
- A node that ends the walk is terminal: `type Outcome = ()`.
- A problem the program handles as part of its flow (bad input, a missing file it can create) is an outcome. A problem that stops the walk is an error.

## Errors

- Cannot fail: `type Error = Infallible`. The walk unwraps it with `let Ok(outcome) = …`.
- Can fail: a `thiserror` enum named `Error`, one variant per cause, with a message that says what could not be done. Propagate with `?`; never swallow with `.ok()`.

## State

- A node asks for state through `Has*` traits in `src/nodes/mod.rs`: `HasCount` has `count()` and `count_mut()`.
- Require only the traits the node touches. Reuse an existing trait before declaring a new one.
- Read and write through the accessors only.

## Utils

- Every side effect goes through `self.utils`: printing, reading input, time, randomness, files, network, processes. The node body itself stays pure and deterministic given its utils.
- One method per effect, in the terms of the domain: `sleep(Duration)`, not `sleep(u64)`; `read_line(prompt)`, not `stdin()`.
- Anything that can fail returns a `Result`.
- An async util returns `impl Future<Output = T> + Send`, and the mock implements it with `async fn`.
- `SystemUtils` implements each method in a line. If that needs a crate, is shared with another node or is long, it belongs in `src/utils/` (`amble help util`).
- A node with no side effects at all may drop `Utils`, `SystemUtils` and the `utils` field, keeping a plain struct that derives `Default`.

## The body

- Keep `run` `Send`: do not hold a `MutexGuard` or other non-`Send` value across an `.await`.
- No ids, no `Transition`, nothing from `crate::walks`.
