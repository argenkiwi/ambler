# ambler

Scaffolds and checks Ambler projects: programs built as walks (state machines) of small, tested nodes. This project is TypeScript (ambler-ts, on Bun); `ambler` worked that out from `ambler.ts`.

    ambler new [<name>] [--example] [--target rust|ts|py]  create a project (no name: in this empty folder)
    ambler node <name> …                  add a node: its contract, a throwing body, failing tests
    ambler util <name>                    add a module of side-effectful functions
    ambler walk <name> --nodes … --edge … wire nodes into a walk, with its test, spec and script
    ambler check [--lint-only]            verify the conventions, then tsc, biome and tests
    ambler pave <spec> --agent '<cmd>'    implement a spec with an agent, one task per run
    ambler help [topic]                   this guide, or one topic

Topics: a verb (new, node, util, walk, check, pave, help), or conventions, node-design, testing, spec, agents.

## The four ideas

- State: one object per walk. `nodes/state.ts` declares a `Has*` interface per field; each node's `State` extends only the ones it uses, and the walk's `State` has them all.
- Node: one step. A factory takes the edges and utils and returns a function from state to `[next, newState]`. It returns the id its edge was wired to, never a node it chose.
- Utils: a node's side effects, one function each, in its `Utils` type. `defaultUtils` is the real thing; tests pass their own.
- Walk: wires each node's edges to the next node in `createWalk`. The edges record is typed, so an unwired edge does not compile.

## Layout

    ambler.ts       NodeFactory, ambler(), walk(), WalkError (identical in every project)
    walks/          one file per walk; each runs itself under `bun walks/<walk>.ts`
    walks/tests/    one wiring test per walk
    nodes/          one file per node; state.ts holds the Has* state interfaces (and only it)
    nodes/tests/    one test file per node
    utils/          side-effectful functions nodes delegate to
    specs/          one plain-language spec per walk

## Usual order of work

1. Describe the walk in `specs/<walk>.md` (`ambler help spec`).
2. For each node: `ambler node`, then tests, then the body (`ambler help node-design`, `ambler help testing`).
3. `ambler walk` to wire them; finish the wiring test (`ambler help walk`).
4. `ambler check`.

Run `bun install` once in a new project, so the checks and the formatter are there.
