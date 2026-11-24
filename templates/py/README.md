# {{project}}

An [Ambler](https://github.com/argenkiwi/ambler-py) project: a program built as walks (state machines) of small, tested nodes, in Python with [uv](https://docs.astral.sh/uv/).

## Layout

```
src/{{package}}/engine.py   Node protocol and Walk base class (written by ambler; leave as is)
src/{{package}}/walks/      Walks: match each node's outcomes to the next node
src/{{package}}/nodes/      Nodes: one step each; the state protocols they share in __init__.py
src/{{package}}/utils/      Side-effectful utilities
tests/nodes/  tests/walks/  pytest suites for the nodes and the walks
specs/                      Plain-language spec for each walk
```

## Development

Install the tools once with `uv sync`. Components are created with [ambler](https://github.com/argenkiwi/ambler) and built test-first:

```sh
ambler help            # conventions and guides
ambler node <name> …   # add a node
ambler walk <name> …   # wire nodes into a walk
ambler check           # conventions, ruff, pyright, pytest
uv run <walk>         # run a walk
```
