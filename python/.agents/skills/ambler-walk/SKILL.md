---
name: ambler-walk
description: Creates a complete Ambler walk in Python — the wiring file (walks/<name>.py) and the Markdown spec (specs/<name>.md) — and ensures all required nodes exist. Use this whenever a user wants to add a new program or flow to an Ambler project, even if they say "new walk", "add a program", "wire up these nodes", or just describe what they want the app to do.
metadata:
  author: leandro
  version: "2.5-py"
---

# Ambler Walk (Python)

This skill guides you in creating a complete Ambler walk. A walk is a state-machine program consisting of two files:

1. `walks/<name>.py` — Python file defining the shared `State`, the initial state, and the wired node graph.
2. `specs/<name>.md` — Markdown specification describing the shared state and the logic/transitions of each step.

All commands are run from the project root (the directory containing `ambler.py`).

---

## Step 1 — Identify the Walk

- Determine the walk name (lowercase snake_case, e.g. `my_walk`). The file will be `walks/<name>.py`.
- Clarify the walk's purpose: what program does it implement?
- Identify the nodes (steps) required and their transitions.

---

## Step 2 — Research and Reuse

Before creating new nodes, research existing ones to promote reuse and consistency:

- **Review Existing Specs:** Browse `specs/*.md` to identify nodes that already implement the required logic (e.g., "Ollama Check", "Model Select"). This is more efficient than reading source code and helps maintain architectural alignment.
- **Check Node Implementation:** For each node identified, verify its existence in `nodes/<node_name>.py`.

---

## Step 3 — Create the Specification File (`specs/<name>.md`)

Create the Markdown spec using the `/ambler-spec` skill. This acts as the **blueprint** for the entire walk, defining the shared state and the logic for every node.

---

## Step 4 — Ensure Nodes Exist

For each node defined in the specification:

- If it does **not** exist in `nodes/<node_name>.py`, create it using the `/ambler-node` skill. This runs a full TDD pass — skeleton, tests (red), implementation, tests (green) — before completing. No separate test step is needed.
- If the node already exists but has no `nodes/test_<node_name>.py`, use the `/ambler-test` skill to retrofit coverage.

---

## Step 5 — Create the Wiring File (`walks/<name>.py`)

```python
import asyncio
import importlib
import inspect
import sys
import os
from typing import Literal, Optional, TypedDict

# Add parent directory to path to import ambler and nodes
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ambler import ambler


class State(TypedDict):
    field: str


NodeId = Literal["START", "NEXT", "STOP"]

# Node modules are imported lazily, the first time each node runs.
amble = ambler({
    "START": lambda: importlib.import_module("nodes.start").factory({"onSuccess": "NEXT", "onError": "START"}),
    "NEXT": lambda: importlib.import_module("nodes.next").factory({"onComplete": "STOP"}),
    "STOP": lambda: importlib.import_module("nodes.stop").factory({"onDone": None}),
})


async def main():
    node_id: Optional[NodeId] = "START"
    state: State = {"field": "initial"}

    while node_id:
        next = amble(node_id, state)
        node_id, state = await next if inspect.isawaitable(next) else next


if __name__ == "__main__":
    asyncio.run(main())
```

**Key rules:**
- Append the project root to `sys.path` before importing, then `from ambler import ambler`.
- Define the `State` `TypedDict` at the top of the file.
- Define a `NodeId` `Literal` of UPPERCASE node identifiers, matching the ALL_CAPS references in the spec.
- Pass `ambler({...})` a dict of node IDs to **zero-argument functions that return nodes**. Each is called once, the first time that node runs, and the result is cached. An unknown node ID raises `ValueError`.
- Import node modules lazily inside the lambda with `importlib.import_module("nodes.<node_name>").factory({...})` — do not import node modules at the top of the file.
- Wire terminal edges to `None` (e.g., `{"onDone": None}`) so the loop ends.
- Create `amble` at module level, outside `main()`, and in the loop `await` only when `inspect.isawaitable(next)` — sync nodes return a tuple directly.

---

## Step 6 — Verify

Run the walk to confirm it behaves as specified:

```bash
python3 walks/<name>.py
```

If the walk has new nodes, also run all node tests:

```bash
python3 -m unittest discover -s nodes -t .
```

---

## Checklist

Before finishing, confirm:

- [ ] `specs/<name>.md` exists and matches the node IDs in the `.py` file.
- [ ] `walks/<name>.py` exists with the correct `State`, `NodeId`, initial state, and wired nodes.
- [ ] Every node used in the walk has a corresponding `nodes/<node_name>.py` and passing `nodes/test_<node_name>.py` (enforced by `/ambler-node`).
- [ ] All tests pass (`python3 -m unittest discover -s nodes -t .`).
- [ ] Any third-party packages the nodes' utils need are in `requirements.txt` (see `/ambler-util`).
- [ ] The walk runs end-to-end without errors.
