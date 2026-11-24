# {{project}}

An [Ambler](https://github.com/argenkiwi/ambler-py) project: a program built as walks (state machines) of small, tested nodes, in Python with [uv](https://docs.astral.sh/uv/).

## Layout

```
src/{{package}}/engine.py   Node protocol and Walk base class (written by amble; leave as is)
src/{{package}}/walks/      Walks: match each node's outcomes to the next node
src/{{package}}/nodes/      Nodes: one step each; the state protocols they share in __init__.py
src/{{package}}/utils/      Side-effectful utilities
tests/nodes/  tests/walks/  pytest suites for the nodes and the walks
specs/                      Plain-language spec for each walk
```

## Development

Install the tools once with `uv sync`. Components are created with [amble](https://github.com/argenkiwi/amble) and built test-first:

```sh
amble help            # conventions and guides
amble node <name> …   # add a node
amble walk <name> …   # wire nodes into a walk
amble check           # conventions, ruff, pyright, pytest
uv run <walk>         # run a walk
```
