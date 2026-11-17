# amble help

    amble help [topic]

Prints a guide on standard output. Without a topic it prints the overview: the verbs, the four ideas, the layout and the usual order of work.

Topics:

- `new`, `node`, `util`, `walk`, `check`, `help`: what each verb takes, writes and leaves to you.
- `conventions`: every convention in one page, with the reason for each.
- `node-design`: how to choose a node's outcomes, errors, state and utils.
- `testing`: test-first nodes, mocks, and wiring tests for walks.
- `spec`: the format of `specs/<walk>.md`.
- `agents`: how `AGENTS.md` and the skill relate to this command, and the text to add to a project that lacks them.

An unknown topic lists the topics on standard error and exits 1. So does an unknown verb.
