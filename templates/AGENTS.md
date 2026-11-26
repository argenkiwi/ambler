# Agent instructions

This is an Ambler project: a program built as walks (state machines) of small, tested nodes. The `ambler` command line creates its components and checks its conventions. `ambler help` explains the layout; `ambler help <topic>` goes deeper.

- **Create with `ambler`, not by hand.** New nodes, walks and utility modules come from `ambler node`, `ambler walk` and `ambler util`. They write the file, register it and wire what can be wired; you then edit what they generated.
- **Read before designing.** Run `ambler help node-design` before choosing a node's outcomes, state and utils, `ambler help walk` before wiring nodes together, and `ambler help spec` before writing a spec.
- **Test first.** A generated node fails its tests with `not yet implemented`. Write the tests, see them fail, then implement the node without touching the tests again. `ambler help testing` has the rules.
- **Keep the spec true.** `specs/<walk>.md` describes each walk in plain language. Change it in the same commit as the wiring.
- **Finish with `ambler check`.** It verifies the conventions, then formatting, lints and tests. The work is not done until it passes.
- **Leave the engine alone.** `src/engine.rs` (Rust), `ambler.ts` (TypeScript) or `src/<package>/engine.py` (Python) is the same in every Ambler project.

If `ambler` is not installed: `cargo install --git https://github.com/argenkiwi/ambler`.
