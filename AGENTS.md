# Agent instructions

This is an Ambler project: a program built as walks (state machines) of small, tested nodes. The `amble` command line creates its components and checks its conventions. `amble help` explains the layout; `amble help <topic>` goes deeper.

- **Create with `amble`, not by hand.** New nodes, walks and utility modules come from `amble node`, `amble walk` and `amble util`. They write the file, register it and wire what can be wired; you then edit what they generated.
- **Read before designing.** Run `amble help node-design` before choosing a node's outcomes, state and utils, `amble help walk` before wiring nodes together, and `amble help spec` before writing a spec.
- **Test first.** A generated node fails its tests with `not yet implemented`. Write the tests, see them fail, then implement `run` without touching the tests again. `amble help testing` has the rules.
- **Keep the spec true.** `specs/<walk>.md` describes each walk in plain language. Change it in the same commit as the wiring.
- **Bump the version on PRs.** Every pull request must increment the package version in `Cargo.toml` (following semver).
- **Finish with `amble check`.** It verifies the conventions, then formatting, lints and tests. The work is not done until it passes.
- **Leave `src/engine.rs` alone.** It is the same in every Ambler project.

If `amble` is not installed: `cargo install --git https://github.com/argenkiwi/amble`.
