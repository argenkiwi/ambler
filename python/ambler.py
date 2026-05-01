from typing import TypeVar, Union, Callable, Dict, Tuple, Optional, Any, Awaitable
import asyncio

S = TypeVar("S")
K = TypeVar("K", bound=str)
H = TypeVar("H", bound=str)

T = TypeVar("T")

# Represents a value that can be either a synchronous value or an Awaitable of that value.
MaybeAwaitable = Union[T, Awaitable[T]]

# A map from edge names to the next node identifier (or None to terminate).
Edges = Dict[H, Optional[K]]

# The result returned by a node: a tuple of [nextNodeId, newState].
Next = Tuple[Optional[K], S]

# A function that represents a node in the state machine.
Node = Callable[[S, K], MaybeAwaitable[Next[S, K]]]

def ambler(nodes: Dict[K, Node[S, K]]) -> Callable[[K, S], MaybeAwaitable[Next[S, K]]]:
    """
    Creates a single-step executor for a node registry.
    """
    def execute(node_id: K, state: S) -> MaybeAwaitable[Next[S, K]]:
        node = nodes.get(node_id)
        if not node:
            raise ValueError(f"Node not found: {node_id}")

        return node(state, node_id)
    
    return execute
