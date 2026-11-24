# {{package}}

An [Ambler](https://github.com/argenkiwi/ambler-rs) project: a program built as walks (state machines) of small, tested nodes.

## Layout

```
src/engine.rs    Node and Walk traits, Transition (written by amble; leave as is)
src/main.rs      Runs the primary walk (extra walks go in src/bin/)
src/walks/       Walks: wire node outcomes to the next node, each with an inline wiring test
src/nodes/       Nodes: one step each, plus the state traits they share
src/utils/       Side-effectful utilities
specs/           Plain-language spec for each walk
```

## Development

Components are created with [amble](https://github.com/argenkiwi/amble) and built test-first:

```sh
amble help            # conventions and guides
amble node <name> …   # add a node
amble walk <name> …   # wire nodes into a walk
amble check           # conventions, fmt, clippy, tests
```
