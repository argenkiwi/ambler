# amble conventions

Every convention of an Ambler project, with why it is there. `amble check` enforces the structural ones.

## Nodes

- A node returns a typed `Outcome`, never the id of the next node. Wiring is the walk's job, so a node can be reused in any walk.
- A terminal node has `type Outcome = ()`. A node that cannot fail has `type Error = Infallible`, and callers write `let Ok(outcome) = …` with no `unwrap`.
- A node file exports `Outcome` (unless terminal), `Error` (if fallible), `Utils`, `SystemUtils` and the struct named after the file.
- Utils are a generic parameter with a default, `Start<U = SystemUtils>`, not a boxed trait object: no allocation, and the mock is a plain type.
- `new()` is implemented on the defaulted type (`impl Start`), `with_utils()` on the generic one (`impl<U> Start<U>`). Default type parameters do not apply in expressions, so `Start::new()` only resolves this way.
- Bounds are `S: HasX + Send` and `U: Utils + Sync`, which keeps `run` a `Send` future.
- State is reached through `Has*` traits declared in `src/nodes/mod.rs`, with a getter and a `_mut` accessor.

## Walks

- `step` is one exhaustive `match` on `Id`, and inside it one exhaustive `match` per node on its `Outcome`. No `_ =>` arm, no map of closures, no panics: a forgotten edge is a compile error.
- A walk module exports only `Error` and `run()`. `State`, `Id` and the struct that owns the nodes are private.
- The walk struct is generic over each node's utils, with `SystemUtils` defaults, so the wiring test can script every node.
- Walks are concrete types. The traits use `async fn`, which is not `dyn`-compatible.

## Tests

- Inline `#[cfg(test)] mod tests` in the file under test, for nodes and walks alike. No `tests/` folder: the wiring test needs the walk's private `State` and `Id`.
- Nodes are built test-first.

## Files

- `src/nodes/<name>.rs`, `src/walks/<name>.rs`, `src/utils/<name>.rs`: snake_case, registered alphabetically in the folder's `mod.rs`.
- `specs/<walk>.md` for every walk.
- `src/main.rs` runs the primary walk; others get `src/bin/<walk>.rs`. Cargo finds both without `[[bin]]` entries.
- `src/engine.rs` holds `Node`, `Transition` and `Walk`, re-exported from the crate root. It is copied in, not a dependency, and is not edited.

## Project

- `unsafe_code` is forbidden and clippy warnings are errors in `amble check`.
- Errors are `thiserror` enums.
