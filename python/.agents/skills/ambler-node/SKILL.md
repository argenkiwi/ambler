---
name: ambler-node
description: Creates a new Ambler node in Python — with tests, using test-driven development. Use this whenever the user wants to add a node, step, or state to an Ambler project — even if they phrase it as "add a step", "create a handler", or describe the behaviour without using the word "node". Also use when creating a node WITH tests or following TDD for a new node. For adding tests to an existing node that already has an implementation, use `/ambler-test` instead.
metadata:
  author: leandro
  version: "3.0-py"
---

# Ambler Node (Python)

Follow these steps to create a new node using test-driven development. Tests are written before the implementation.

All commands are run from the project root (the directory containing `ambler.py`).

## 1. Gather requirements

Before writing any code, determine:

- **Research Existing Nodes:** Check `specs/*.md` and `nodes/` to see if a node with similar behaviour already exists. It is always better to reuse or adapt an existing node than to create a duplicate.
- **Node name**: The purpose of the node (e.g., `retry`, `prompt`, `validate`). The file will be named `nodes/<name>.py` (snake_case, no `_node` suffix).
- **State shape**: What fields does this node read or mutate? Every node has a minimum `State` `TypedDict` that must include the fields it touches.
- **Edges**: What named transitions can this node take? Define an `Edge` type (a `Literal` of strings) for these names.
- **Utils**: What side-effectful operations does the node perform? List them (e.g., `print`, `read_line`, `sleep`, `random`). Each becomes a method on the `Utils` `Protocol` with a production implementation in `DefaultUtils`.
- **Sync vs async**: Does the node need `await`? Determines whether the factory returns a plain `def node(state)` or an `async def node(state)`.
- **Behaviour**: What does the node do, and how does it choose which Edge to follow?
- **Branches**: List every edge return the node can take (e.g., `onSuccess`, `onError`). Each becomes one test case. The types and branches decided here are the contract — if you find you need to change them during implementation, restart from this step.

---

## 2. Create a skeleton `nodes/<name>.py`

Write only the type exports and a stub factory. The node body must raise so that tests fail in step 4.

**Synchronous skeleton:**

```python
from typing import Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    # Fields from requirements
    count: int


Edge = Literal["onSuccess", "onError"]  # from requirements


class Utils(Protocol):
    def print(self, message: str) -> None: ...


class DefaultUtils:
    def print(self, message: str) -> None:
        print(message)


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    def node(state: State):
        raise NotImplementedError("not implemented")

    return node
```

**Asynchronous skeleton:**

```python
import asyncio
from typing import Awaitable, Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    count: int


Edge = Literal["onSuccess", "onError"]


class Utils(Protocol):
    def sleep(self, ms: int) -> Awaitable[None]: ...


class DefaultUtils:
    async def sleep(self, ms: int) -> None:
        await asyncio.sleep(ms / 1000)


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    async def node(state: State):
        raise NotImplementedError("not implemented")

    return node
```

Verify the skeleton imports cleanly before writing tests:

```bash
python3 -c "import nodes.<name>"
```

---

## 3. Write `nodes/test_<name>.py`

Write one test method per branch identified in step 1. All utils must be mocked — no real I/O, network, or timing.

**Synchronous node** (`unittest.TestCase`):

```python
import unittest
from nodes.<name> import factory


class MockUtils:
    def print(self, message):
        pass


class Test<Name>Node(unittest.TestCase):
    def test_<behaviour>_when_<condition>(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils())
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
    async def sleep(self, ms):
        pass


class Test<Name>Node(unittest.IsolatedAsyncioTestCase):
    async def test_<behaviour>_when_<condition>(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils())
        next_id, next_state = await node({"count": 0})

        self.assertEqual(next_id, "NEXT")
        self.assertEqual(next_state["count"], expected_value)


if __name__ == "__main__":
    unittest.main()
```

### Test rules

- **One test per branch** — one for each edge the node can return.
- **Mock all `Utils`** — a `MockUtils` class implementing every `Utils` method deterministically, with no side effects. Pass per-test values through its constructor (e.g., `MockUtils("42")`) or record calls on it (e.g., `self.message = message`) to assert on them.
- **Unpack the result** as `next_id, next_state` and assert on both. For an edge wired to `None`, use `self.assertIsNone(next_id)`.
- **Edge targets**: Use placeholder node IDs such as `"NEXT"` and `"ERROR"` (or `None` for terminal edges) in the `edges` map.
- **Test names**: `class Test<Name>Node` with methods named `test_<expected_behaviour>_when_<condition>`.
- **Do not use `IsolatedAsyncioTestCase` or `await` for synchronous nodes** — it masks a node accidentally returning a coroutine.

---

## 4. Run tests — expect failure

```bash
python3 -m unittest nodes.test_<name>
```

Tests must fail with `NotImplementedError: not implemented` (or assertion errors). This confirms the tests are actually exercising the code path.

- If tests fail with `NotImplementedError: not implemented` → correct, proceed to step 5.
- If tests fail with an **`ImportError`, `AttributeError` or `TypeError`** → the skeleton doesn't match the test — fix the skeleton, not the tests.
- If tests **pass** → the skeleton was too complete; ensure the node raises.

---

## 5. Implement `nodes/<name>.py`

Replace the stub body with the real logic. The type exports (`State`, `Edge`, `Utils`, `DefaultUtils`) stay exactly as written in step 2.

```python
from typing import Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    count: int


Edge = Literal["onSuccess", "onError"]


class Utils(Protocol):
    def print(self, message: str) -> None: ...


class DefaultUtils:
    def print(self, message: str) -> None:
        print(message)


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    def node(state: State):
        # Use 'async def node' if using await.
        # Always build a new dict when updating: {**state, "field": new_value}
        next_state = {**state, "count": state["count"] + 1}
        return edges["onSuccess"], next_state

    return node
```

### Key rules

- **Signature**: Nodes take only `state` and return a `(next_id, next_state)` tuple. The factory is always `def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]`.
- **Default utils**: Use `utils = utils or DefaultUtils()` inside the factory. Never write `utils: Utils = DefaultUtils()` in the signature — default arguments are evaluated once at import time and shared between calls.
- **Immutability**: Never mutate `state` directly; always return `{**state, **updates}`.
- **Utils**: All side effects go through `utils.*`. Complex or reusable logic (e.g., LLM calls, file I/O) should be moved to `utils/` via `/ambler-util`.
- **Termination**: Terminal nodes are wired with `None` in the walk (e.g., `factory({"onDone": None})`).
- **Do not modify the tests** between steps 4 and 6 — the tests define the contract; the implementation must satisfy them.

---

## 5a. Adapter Nodes (Optional)

If you need to use an existing node but the walk's state has different property names, create an adapter node. Adapters wrap a real factory, so there is no stub phase — write the adapter implementation and tests together, then verify green in step 6.

```python
import inspect
from typing import Optional, TypedDict, TypeVar
from ambler import Edges, Node
from nodes.original import Edge, Utils, factory as original_factory

K = TypeVar("K", bound=str)


class State(TypedDict):
    new_property_name: int


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    node = original_factory(edges, utils)

    async def adapter(state: State):
        result = node({"original_property_name": state["new_property_name"]})
        next_id, next_state = await result if inspect.isawaitable(result) else result

        return next_id, {**state, "new_property_name": next_state["original_property_name"]}

    return adapter
```

---

## 6. Run tests — must pass

```bash
python3 -m unittest nodes.test_<name>
```

All tests must be green. Fix the implementation (never the tests) until they pass.

---

## 7. Checklist before finishing

- [ ] `nodes/<name>.py` uses the `Edge` naming convention for edge keys (`onSuccess`, `onError`, …).
- [ ] Factory is exported as `factory` with the signature `(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]` and uses `utils = utils or DefaultUtils()`.
- [ ] `State` `TypedDict` is minimal.
- [ ] `DefaultUtils` provides real implementations.
- [ ] No direct state mutation.
- [ ] Shared logic is in `utils/`.
- [ ] `nodes/test_<name>.py` has one test per branch.
- [ ] All `Utils` in tests are mocked.
- [ ] Tests were written (step 3) before the full implementation (step 5).
- [ ] Tests failed on step 4 before passing on step 6.
- [ ] No tests were modified between steps 4 and 6.
