name: ambler-node
description: Creates a new Ambler node in the nodes/ directory. Use this whenever the user wants to add a node, step, or state to an Ambler project — even if they phrase it as "add a step", "create a handler", or describe the behavior without using the word "node".
metadata:
  author: leandro
  version: "1.2-py"

# Ambler Node (Python)

Follow these steps to create a new node in the `nodes/` directory.

## 1. Gather requirements

Before writing any code, determine:

- **Node name**: The purpose of the node (e.g., `retry`, `prompt`, `validate`). The file will be named `<name>_node.py`.
- **State shape**: What fields does this node read or mutate? Every node has a minimum `State` (Dict[str, Any]) that must include the fields it touches.
- **Hook (Edges)**: What named transitions can this node take? Define a `Hook` type (union of strings) for these names.
- **Utils**: What side-effectful operations does the node perform? List them (e.g., `print`, `readLine`). Each becomes a method on the `Utils` Protocol.
- **Behavior**: What does the node do, and how does it choose which Hook to follow?

---

## 2. Create `nodes/<name>_node.py`

Use the following structure exactly. Adhere to naming conventions.

```python
from typing import Dict, Any, Optional, Protocol, Union, Awaitable
from ambler import Edges, Node, Next

class Utils(Protocol):
    def log(self, message: str) -> None:
        ...
    # def read_line(self, prompt: str) -> Optional[str]:
    #     ...

class DefaultUtils:
    def log(self, message: str) -> None:
        print(message)
    # def read_line(self, prompt: str) -> Optional[str]:
    #     try: return input(prompt)
    #     except EOFError: return None

# Edge = Union[str, str] # "onSuccess" | "onError"

def create(
    edges: Edges[str, str],
    utils: Utils = DefaultUtils()
) -> Node[Dict[str, Any], str]:
    def node_fn(state: Dict[str, Any], key: str) -> Next[Dict[str, Any], str]:
        # Node logic here.
        # Use 'async def' and 'await' if needed.
        
        # Always return a new dict when updating: {**state, "field": newValue}
        next_state = {**state, "count": state.get("count", 0) + 1}
        
        # Return (edges["onHookName"], next_state) to transition.
        return edges["onSuccess"], next_state

    return node_fn
```

### Key rules

- **Imports**: Always import `Edges`, `Node`, `Next` from `ambler`.
- **Naming**: Use `snake_case` for filenames and functions.
- **Utils**: Use `Protocol` for the `Utils` interface and a `DefaultUtils` class for production.
- **Immutability**: Never mutate `state` directly; always return a new dict: `{**state, **updates}`.
- **Termination**: Nodes that terminate the walk still use `Edges` in their `create` signature. In the `walks/*.py` file, they are initialized with an edge mapped to `None`.

---

## 3. Create `nodes/test_<name>_node.py`

Use the `/ambler-test` skill to generate the test file.

---

## 4. Checklist before finishing

- [ ] `nodes/<name>_node.py` uses `snake_case`.
- [ ] `create` function returns a `node_fn` that matches the `Node` type.
- [ ] `DefaultUtils` provides real implementations.
- [ ] No direct state mutation.
- [ ] Tests exist in `nodes/test_<name>_node.py`.
