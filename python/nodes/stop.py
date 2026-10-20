from typing import Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    count: int


Edge = Literal["onDone"]


class Utils(Protocol):
    def print(self, message: str) -> None: ...


class DefaultUtils:
    def print(self, message: str) -> None:
        print(message)


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    def node(state: State):
        utils.print(f"Final count: {state['count']}")
        return edges["onDone"], state

    return node
