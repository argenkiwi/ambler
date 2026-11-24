# {{package}}

An [Ambler](https://github.com/argenkiwi/ambler-ts) project: a program built as walks (state machines) of small, tested nodes, in TypeScript on [Bun](https://bun.sh/).

## Layout

```
ambler.ts        The engine: NodeFactory, ambler(), walk() (written by ambler; leave as is)
walks/           Walks: wire each node's edges to the next node; tests in walks/tests/
nodes/           Nodes: one step each, a factory of State, Edge and Utils; tests in nodes/tests/
utils/           Side-effectful utilities
specs/           Plain-language spec for each walk
```

## Development

Install the tools once with `bun install`. Components are created with [ambler](https://github.com/argenkiwi/ambler) and built test-first:

```sh
ambler help            # conventions and guides
ambler node <name> …   # add a node
ambler walk <name> …   # wire nodes into a walk
ambler check           # conventions, tsc, biome, tests
bun run <walk>        # run a walk
```
