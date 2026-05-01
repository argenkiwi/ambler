name: ambler-test
description: Creates a test file for an Ambler node in the nodes/ directory. Use this whenever the user wants tests for a node — including "test this node", "add tests", "write tests for X", or any time a node is created without a corresponding test file.
metadata:
  author: leandro
  version: "1.2-py"

# Ambler Test (Python)

Follow these steps to create a test file for a node in the `nodes/` directory.

## 1. Gather requirements

Before writing any code, determine:

- **Node name**: The snake_case name (e.g., `retry`, `prompt`, `validate`) — the test file will be `nodes/test_<name>_node.py`.
- **Node's State, Edges, and Utils**: Read `nodes/<name>_node.py` to understand what the node does, which edges it has, and what utils it uses.
- **Branches to cover**: Every `return edges["onEdgeName"], ...` line is one branch; terminal nodes return `None, state`. List them all before writing any test.

---

## 2. Determine sync vs async

Look at the node's `create` return function:

- `def node_fn(state, key): ...` — **synchronous**: tests use `unittest.TestCase`.
- `async def node_fn(state, key): ...` — **asynchronous**: tests use `unittest.IsolatedAsyncioTestCase`.

---

## 3. Create `nodes/test_<name>_node.py`

Write one test method per meaningful branch of logic.

**Synchronous node:**

```python
import unittest
from <name>_node import create

class Test<Name>Node(unittest.TestCase):
    def test_<name>_node_should_<behavior>_when_<condition>(self):
        initial_state = { "count": 0 }
        
        class MockUtils:
            def log(self, message): pass
            # Override each util to be deterministic and side-effect-free.

        node = create({"onSuccess": "next"}, MockUtils())
        next_id, next_state = node(initial_state, "start")

        self.assertEqual(next_id, "next")
        self.assertEqual(next_state["some_field"], value)
```

**Asynchronous node:**

```python
import unittest
from <name>_node import create

class Test<Name>Node(unittest.IsolatedAsyncioTestCase):
    async def test_<name>_node_should_<behavior>_when_<condition>(self):
        initial_state = { "count": 0 }
        
        class MockUtils:
            def log(self, message): pass
            async def sleep(self, ms): pass
            # Override each util to be deterministic and side-effect-free.

        node = create({"onSuccess": "next"}, MockUtils())
        next_id, next_state = await node(initial_state, "start")

        self.assertEqual(next_id, "next")
        self.assertEqual(next_state["some_field"], value)
```

---

## 4. Checklist before finishing

- [ ] `nodes/test_<name>_node.py` exists with one test per branch.
- [ ] All `Utils` are mocked.
- [ ] Every edge path has a dedicated test.
- [ ] Run `python3 -m unittest nodes/test_<name>_node.py` to verify.
