---
name: ambler-test
description: Adds or updates tests for an existing Ambler node in Python that already has an implementation. Use this when a node exists in nodes/<name>.py but has no nodes/test_<name>.py, when tests need extending after new branches were added, or when retrofitting TDD onto existing code. For creating a brand-new node with tests, use `/ambler-node` instead.
metadata:
  author: leandro
  version: "3.0-py"
---

# Ambler Test (Python)

Follow these steps to add or update tests for an existing node in `nodes/`.

> For brand-new nodes, use `/ambler-node` — it creates both the implementation and tests via a TDD cycle. This skill is for retrofitting tests onto nodes that already exist.

All commands are run from the project root (the directory containing `ambler.py`).

---

## 1. Read the existing node

Read `nodes/<name>.py` in full. Identify:

- **`State`**, **`Edge`**, and **`Utils`** types.
- **Sync vs async**: Does the factory return `def node(state)` or `async def node(state)`?
- **All branches**: Every `return edges["onEdgeName"], ...` line is one branch. List them all — each becomes a test.

---

## 2. Check for existing tests

Check whether `nodes/test_<name>.py` exists.

- If it **does not exist**: write a complete new test file covering every branch.
- If it **does exist**: read it, identify which branches already have coverage, and list the gaps. Only add tests for uncovered branches — do not overwrite passing tests unless the node's interface changed.

---

## 3. Write or update `nodes/test_<name>.py`

Write one test method per uncovered branch. All utils must be mocked.

**Synchronous node** (`unittest.TestCase`):

```python
import unittest
from nodes.<name> import factory


class MockUtils:
    def __init__(self, input):
        self.input = input

    def read_line(self, message):
        return self.input

    def print(self, message):
        pass
    # Implement each util to be deterministic and side-effect-free.


class Test<Name>Node(unittest.TestCase):
    def test_<behaviour>_when_<condition>(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils("42"))
        next_id, next_state = node({"count": 0})

        self.assertEqual(next_id, "NEXT")
        self.assertEqual(next_state["count"], expected_value)


if __name__ == "__main__":
    unittest.main()
```

**Asynchronous node** (`unittest.IsolatedAsyncioTestCase`):

```python
import unittest
from nodes.<name> import factory


class MockUtils:
    def __init__(self, random):
        self._random = random

    def print(self, message):
        pass

    async def sleep(self, ms):
        pass

    def random(self):
        return self._random
    # Implement each util to be deterministic and side-effect-free.


class Test<Name>Node(unittest.IsolatedAsyncioTestCase):
    async def test_<behaviour>_when_<condition>(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils(0.6))
        next_id, next_state = await node({"count": 0})

        self.assertEqual(next_id, "NEXT")
        self.assertEqual(next_state["count"], expected_value)


if __name__ == "__main__":
    unittest.main()
```

### Test rules

- **One test per branch** — cover every `edges["onEdgeName"]` return.
- **Mock all `Utils`** — a `MockUtils` class implementing every method of the node's `Utils` `Protocol`; no real I/O, network, or timing. Async utils (e.g., `sleep`) must stay `async def` in the mock.
- **Unpack the result** as `next_id, next_state` and assert on both. For an edge wired to `None`, use `self.assertIsNone(next_id)`.
- **Edge targets**: Use placeholder node IDs such as `"NEXT"` and `"ERROR"` (or `None` for terminal edges) in the `edges` map.
- **Test names**: `class Test<Name>Node` with methods named `test_<expected_behaviour>_when_<condition>`.
- **Do not use `IsolatedAsyncioTestCase` or `await` for synchronous nodes** — it masks a node accidentally returning a coroutine.

---

## 4. Run tests — must pass

```bash
python3 -m unittest nodes.test_<name>
```

All tests must pass. If any fail, fix the implementation (not the tests).

To run every node test:

```bash
python3 -m unittest discover -s nodes -t .
```

---

## 5. Checklist before finishing

- [ ] `nodes/test_<name>.py` has one test per branch.
- [ ] All `Utils` are mocked — no real I/O, network, or timing.
- [ ] Every edge path has a dedicated test.
- [ ] Test methods follow `test_<expected_behaviour>_when_<condition>`.
- [ ] All tests pass.
