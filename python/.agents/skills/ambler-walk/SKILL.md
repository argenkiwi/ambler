name: ambler-walk
description: Creates a complete Ambler walk — the Python wiring file (walks/<name>.py) and the Markdown spec (specs/<name>.md) — and ensures all required nodes exist. Use this whenever a user wants to add a new program or flow to an Ambler project, even if they say "new walk", "add a program", "wire up these nodes", or just describe what they want the app to do.
metadata:
  author: leandro
  version: "1.1-py"

# Ambler Walk (Python)

This skill guides you in creating a complete Ambler walk. A walk is a state-machine program consisting of two files:

1. `walks/<name>.py` — Python file defining the wiring and execution loop.
2. `specs/<name>.md` — Markdown specification describing the shared state and logic.

---

## Step 1 — Identify the Walk

- Determine the walk name (snake_case, e.g. `my_walk`). The file will be `walks/<name>.py`.
- Identify the nodes required and their transitions.

---

## Step 2 — Ensure Nodes Exist

- Check if `nodes/<node_name>_node.py` exists.
- If not, create it using `/ambler-node`.
- Ensure tests exist using `/ambler-test`.

---

## Step 3 — Create Specification (`specs/<name>.md`)

Use `/ambler-spec`.

---

## Step 4 — Create Wiring File (`walks/<name>.py`)

```python
import asyncio
import inspect
import sys
import os

# Ensure imports work
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ambler import ambler
from nodes import node_a, node_b

async def main():
    nodes = {
        "start": node_a.create({"onSuccess": "next"}),
        "next":  node_b.create({"onDone": None}),
    }

    amble = ambler(nodes)

    node_id = "start"
    state = {"field": "initial"}

    while node_id:
        result = amble(node_id, state)
        if inspect.isawaitable(result):
            node_id, state = await result
        else:
            node_id, state = result

if __name__ == "__main__":
    asyncio.run(main())
```

---

## Step 5 — Verify

Run the walk:
```
python3 walks/<name>.py
```
Run tests:
```
python3 -m unittest discover nodes/
```
