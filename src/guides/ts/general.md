# amble

Scaffolds and checks Ambler projects: programs built as walks (state machines) of small, tested nodes. This project is TypeScript (ambler-ts, on Bun); `amble` worked that out from `ambler.ts`.

    amble new [<name>] [--example] [--target rust|ts|py]  create a project (no name: in this empty folder)
    amble node <name> …                  add a node: its contract, a throwing body, failing tests
    amble util <name>                    add a module of side-effectful functions
    amble walk <name> --nodes … --edge … wire nodes into a walk, with its test, spec and script
    amble check [--lint-only]            verify the conventions, then tsc, biome and tests
    amble pave <spec> --agent '<cmd>'    implement a spec with an agent, one task per run
    amble help [topic]                   this guide, or one topic

Topics: a verb (new, node, util, walk, check, pave, help), or conventions, node-design, testing, spec, agents.

## The four ideas

- State: one object per walk. Each node declares only the fields it uses in its own `State`; the walk's `State` has them all.
- Node: one step. A factory takes the edges and utils and returns a function from state to `[next, newState]`. It returns the id its edge was wired to, never a node it chose.
- Utils: a node's side effects, one function each, in its `Utils` type. `defaultUtils` is the real thing; tests pass their own.
- Walk: wires each node's edges to the next node in `createWalk`. The edges record is typed, so an unwired edge does not compile.

## Layout

    ambler.ts       NodeFactory, ambler(), walk(), WalkError (identical in every project)
    walks/          one file per walk; each runs itself under `bun walks/<walk>.ts`
    walks/tests/    one wiring test per walk
    nodes/          one file per node
    nodes/tests/    one test file per node
    utils/          side-effectful functions nodes delegate to
    specs/          one plain-language spec per walk

## Usual order of work

1. Describe the walk in `specs/<walk>.md` (`amble help spec`).
2. For each node: `amble node`, then tests, then the body (`amble help node-design`, `amble help testing`).
3. `amble walk` to wire them; finish the wiring test (`amble help walk`).
4. `amble check`.

Run `bun install` once in a new project, so the checks and the formatter are there.
