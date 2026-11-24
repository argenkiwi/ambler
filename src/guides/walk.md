# amble walk

    amble walk <name> --nodes <start,next,...> [--edge <node>.<Outcome>=<node|halt>]... [--no-runner]

Wires existing nodes into `src/walks/<name>.rs`. Run it from the project root, after the nodes exist: it reads their outcomes, errors, state traits and utils from `src/nodes/`. If a node is missing it lists the `amble node` commands to run first.

- `--nodes start,count,stop`: the nodes of the walk. The first is where it starts.
- `--edge start.Success=count`: where an outcome leads. Repeat it, or separate edges with commas. `=halt` ends the walk on that outcome. Outcome names may be given in snake_case.
- Edges you leave out become `todo!("wire …")` arms: the walk compiles, and you fill them in.
- `--no-runner`: do not write a binary, for a program whose `main` chooses between walks itself.

What it writes:

- `src/walks/<name>.rs`, registered in `src/walks/mod.rs`:
  - `Error`: `Infallible` if no node can fail, an alias if one can, a `thiserror` enum with `#[from]` if several can.
  - A private `State` holding the fields of every required `Has*` trait, with the trait impls.
  - A private `Id` enum, one variant per node.
  - A private struct that owns the nodes, generic over each node's utils with `SystemUtils` as defaults.
  - `impl Walk` whose `step` matches every outcome of every node.
  - `pub async fn run()`, which starts the walk from a default state.
  - A wiring test with a `Script` that stands in for every node's utils.
- `specs/<name>.md`, if there is none, with a section per node (`amble help spec`).
- The runner: `src/main.rs` if the project has none, otherwise `src/bin/<name>.rs` (run it with `cargo run --bin <name>`).

## Rules for a walk

- Only `Error` and `run` are public. A runner names nothing else.
- Match outcomes by name. Never add a `_ =>` arm: a new outcome must fail to compile until it is wired.
- Use the `run` the `Walk` trait provides. Do not write the loop yourself.
- An infallible node is unwrapped with `let Ok(outcome) = …`; a fallible one with `?`.
- A terminal node maps to `Transition::Halt`.
- `run()` builds the initial state. If the program takes arguments, read `std::env::args()` there, or give `run` a parameter and pass them from `main`.
- State the nodes do not share does not belong in `State`.

## After generating

1. Wire the remaining `todo!` arms.
2. Finish `Script` and the wiring test, one test per path (`amble help testing`).
3. Fill in the spec.
4. Run it, then `amble check`.

Changing a walk later (adding a node, moving an edge) is an edit by hand: add the import, the `Id` variant, the field, the generic parameter and the arm; the compiler points at each one you miss. Update the spec in the same change.
