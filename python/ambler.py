from typing import TypeVar, Union, Callable, Dict, Tuple, Optional, Awaitable

S = TypeVar("S")
K = TypeVar("K", bound=str)
E = TypeVar("E", bound=str)

T = TypeVar("T")

# Represents a value that can be either a synchronous value or an Awaitable of that value.
MaybeAwaitable = Union[T, Awaitable[T]]

# The result of a node's execution: the next node ID (or None to stop) and the updated state.
Next = Tuple[Optional[K], S]

# A processing step in the state machine.
# Receives the current state and returns a Next tuple (or an Awaitable resolving to one).
Node = Callable[[S], MaybeAwaitable[Next[K, S]]]

# A map of edge names to their target node IDs in the walk (or None to stop).
Edges = Dict[E, Optional[K]]


def ambler(nodes: Dict[K, Callable[[], Node[S, K]]]) -> Callable[[K, S], MaybeAwaitable[Next[K, S]]]:
    """
    Creates a state machine runner from a map of node factories.

    Each factory is called once on first use; the resulting node is cached for subsequent calls.

    Raises:
        ValueError: If `node_id` is not present in `nodes`.
    """
    loaded: Dict[K, Node[S, K]] = {}

    def execute(node_id: K, state: S) -> MaybeAwaitable[Next[K, S]]:
        if node_id not in nodes:
            raise ValueError(f"Node not found: {node_id}")
        if node_id not in loaded:
            loaded[node_id] = nodes[node_id]()
        return loaded[node_id](state)

    return execute
