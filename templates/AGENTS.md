# Agent instructions

This is an Ambler project: a program built as walks (state machines) of small, tested nodes. The project's own command line, `<cli>`, creates its components and checks its conventions. `<cli> help` explains the layout; `<cli> help <topic>` goes deeper.

- **Create with `<cli>`, not by hand.** New nodes, walks and utility modules come from `<cli> node`, `<cli> walk` and `<cli> util`. They write the file, register it and wire what can be wired; you then edit what they generated.
- **Read before designing.** Run `<cli> help node-design` before choosing a node's outcomes, state and utils, `<cli> help walk` before wiring nodes together, and `<cli> help spec` before writing a spec.
- **Test first.** A generated node fails its tests with `not yet implemented`. Write the tests, see them fail, then implement the node without touching the tests again. `<cli> help testing` has the rules.
- **Keep the spec true.** `specs/<walk>.md` describes each walk in plain language. Change it in the same commit as the wiring.
- **Finish with `<cli> check`.** It verifies the conventions, then formatting, lints and tests. The work is not done until it passes.
- **Leave the engine alone.** `<engine>` is the same in every Ambler project.

`<cli>` is a dev dependency of this project, so nothing else has to be installed. If it is missing, run `<setup>`.
