# amble check

    amble check [--lint-only]

Run it from the project root when the work is done. It checks the structural conventions and prints one line per violation:

    <path>:<line>: <rule>: <what to do>

If there are none, it runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, stopping at the first that fails. `--lint-only` skips those. It exits 1 if anything failed.

## Rules

- `layout`: `Cargo.toml`, `src/nodes/mod.rs`, `src/walks/mod.rs` and `src/utils/mod.rs` exist.
- `registered`: every file in `src/nodes/`, `src/walks/` and `src/utils/` is declared in its `mod.rs`.
- `sorted`: the `pub mod` lines of each `mod.rs` are alphabetical.
- `parse`: the file is valid Rust.
- `todo`: no `todo!()` is left in a node, walk or utility.
- `node-shape`: a node file has a struct named after it, an `impl Node`, and a `SystemUtils` if it has `Utils`.
- `node-tests`: a node has an inline tests module.
- `walk-surface`: a walk's only public items are `Error` and `run`.
- `walk-tests`: a walk has an inline tests module.
- `wildcard-arm`: `step` has no `_ =>` arm.
- `walk-runner`: `src/main.rs` or a file in `src/bin/` runs the walk.
- `walk-spec`: the walk has `specs/<walk>.md`.
- `spec-nodes`: the spec's `###` headings are exactly the walk's `Id` variants.
- `spec-sections`: a spec has `# Program Specifications`, `## Nodes` and `## Shared State`, and no other sections.
- `util-imports`: a utility imports nothing from `crate::nodes`, `crate::walks` or the engine.

What it cannot check is yours: that outcomes name what happened, that tests cover every branch, that the spec's prose matches the code. See `amble help node-design`, `amble help testing` and `amble help spec`.
