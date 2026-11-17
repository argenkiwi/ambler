# amble node

    amble node <name> (--outcomes <a,b> | --terminal) [--fallible] [--state <field>[:<Type>]]...

Writes `src/nodes/<name>.rs` and registers it in `src/nodes/mod.rs`. Run it from the project root. Decide the flags with `amble help node-design` first: they are the node's contract.

- `<name>`: snake_case, no `_node` suffix. The struct is the same name in PascalCase.
- `--outcomes success,invalid_input`: one variant of `Outcome` per thing that can happen. The walk gives each one a successor.
- `--terminal`: the node ends the walk; `type Outcome = ()`. Give exactly one of `--outcomes` and `--terminal`.
- `--fallible`: adds a `thiserror` `Error` enum, seeded with an `Io` variant. Without it the node's error is `Infallible`.
- `--state count`: the node needs the existing `HasCount` trait from `src/nodes/mod.rs`.
- `--state name:String`: declares a new `HasName` trait there (a getter and `name_mut`) and requires it. Repeat `--state` for more fields.

What you get:

- `Outcome`, `Error` if fallible, an empty `Utils` trait with its `SystemUtils`, and the node struct with `new()` and `with_utils()`.
- `impl Node` with the state bounds and a `todo!()` body.
- A tests module with `MockState`, `MockUtils` and one test per outcome and per error. They fail with `not yet implemented`.

What is left to you, in this order:

1. Give `Utils` one method per side effect and implement each in `SystemUtils`.
2. Finish the tests: rename them, set up state and mocks, assert outcome and state (`amble help testing`).
3. Run `cargo test nodes::<name>::` and see them fail with `not yet implemented`.
4. Implement `run`. Do not change the tests while doing so.
5. Replace the `TODO` doc comments.

It never overwrites: it fails if the file or the registration exists.
