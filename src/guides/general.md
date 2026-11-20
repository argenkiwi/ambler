# amble

Scaffolds and checks Ambler projects: programs built as walks (state machines) of small, tested nodes.

    amble new [<name>] [--example]       create a project (no name: in this empty folder)
    amble node <name> …                  add a node: its contract, a todo body, failing tests
    amble util <name>                    add a module of side-effectful functions
    amble walk <name> --nodes … --edge … wire nodes into a walk, with its spec and runner
    amble check [--no-cargo]             verify the conventions, then fmt, clippy and tests
    amble pave <spec> --agent '<cmd>'    implement a spec with an agent, one task per run
    amble help [topic]                   this guide, or one topic

Topics: a verb (new, node, util, walk, check, pave, help), or conventions, node-design, testing, spec, agents.

## The four ideas

- State: one struct per walk. Nodes see it only through `Has*` traits, so they stay reusable.
- Node: one step. It reads and changes the state, performs side effects through its `Utils`, and returns an `Outcome`. It never names the node that comes next.
- Utils: a node's side effects, one trait method each. `SystemUtils` is the real thing; tests pass a mock.
- Walk: owns the nodes and matches each outcome to the next node. The match is exhaustive, so an unwired outcome does not compile.

## Layout

    src/engine.rs   Node and Walk traits, Transition (identical in every project)
    src/main.rs     runs the primary walk (other walks: src/bin/<walk>.rs)
    src/walks/      one file per walk, with an inline wiring test
    src/nodes/      one file per node; mod.rs also holds the Has* state traits
    src/utils/      side-effectful functions nodes delegate to
    specs/          one plain-language spec per walk

## Usual order of work

1. Describe the walk in `specs/<walk>.md` (`amble help spec`).
2. For each node: `amble node`, then tests, then the body (`amble help node-design`, `amble help testing`).
3. `amble walk` to wire them; finish the wiring test (`amble help walk`).
4. `amble check`.
