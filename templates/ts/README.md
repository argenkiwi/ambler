# {{package}}

An [Ambler](https://github.com/argenkiwi/ambler-ts) project: a program built as walks (state machines) of small, tested nodes, in TypeScript on [Bun](https://bun.sh/).

## Layout

```
ambler.ts        The engine: NodeFactory, ambler(), walk() (written by amble; leave as is)
walks/           Walks: wire each node's edges to the next node; tests in walks/tests/
nodes/           Nodes: one step each, a factory of State, Edge and Utils; tests in nodes/tests/
utils/           Side-effectful utilities
specs/           Plain-language spec for each walk
```

## Development

Install the tools once with `bun install`. Components are created with [amble](https://github.com/argenkiwi/amble) and built test-first:

```sh
amble help            # conventions and guides
amble node <name> …   # add a node
amble walk <name> …   # wire nodes into a walk
amble check           # conventions, tsc, biome, tests
bun run <walk>        # run a walk
```
