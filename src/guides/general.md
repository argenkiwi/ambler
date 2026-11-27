# ambler

Scaffolds and checks Ambler projects: programs built as walks (state machines) of small, tested nodes.

    ambler new [<name>] [--example] [--target rust|ts|py]  create a project (no name: in this empty folder)
    ambler node <name> …                  add a node: its contract, a todo body, failing tests
    ambler util <name>                    add a module of side-effectful functions
    ambler walk <name> --nodes … --edge … wire nodes into a walk, with its spec and runner
    ambler check [--lint-only]             verify the conventions, then fmt, clippy and tests
    ambler pave <spec> --agent '<cmd>'    implement a spec with an agent, one task per run
    ambler help [topic]                   this guide, or one topic

Topics: a verb (new, node, util, walk, check, pave, help), or conventions, node-design, testing, spec, agents.

## The four ideas

- State: one struct per walk. Nodes see it only through `Has*` traits, one field each, so they stay reusable.
- Node: one step. It reads and changes the state, performs side effects through its `Utils`, and returns an `Outcome`. It never names the node that comes next.
- Utils: a node's side effects, one trait method each. `SystemUtils` is the real thing; tests pass a mock.
- Walk: owns the nodes and matches each outcome to the next node. The match is exhaustive, so an unwired outcome does not compile.

## Layout

    src/engine.rs   Node and Walk traits, Transition (identical in every project)
    src/main.rs     runs the primary walk (other walks: src/bin/<walk>.rs)
    src/walks/      one file per walk, with an inline wiring test
    src/nodes/      one file per node; mod.rs also holds the Has* state traits (and only it)
    src/utils/      side-effectful functions nodes delegate to
    specs/          one plain-language spec per walk

## Usual order of work

1. Describe the walk in `specs/<walk>.md` (`ambler help spec`).
2. For each node: `ambler node`, then tests, then the body (`ambler help node-design`, `ambler help testing`).
3. `ambler walk` to wire them; finish the wiring test (`ambler help walk`).
4. `ambler check`.
