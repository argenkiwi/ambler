from typing import Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    count: int


Edge = Literal["onSuccess", "onError"]


class Utils(Protocol):
    def read_line(self, message: str) -> Optional[str]: ...
    def print(self, message: str) -> None: ...


class DefaultUtils:
    def read_line(self, message: str) -> Optional[str]:
        try:
            return input(message)
        except EOFError:
            return None

    def print(self, message: str) -> None:
        print(message)


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    def node(state: State):
        user_input = utils.read_line("Enter a starting number: ")

        if user_input is None or user_input == "":
            return edges["onSuccess"], {**state, "count": 0}

        try:
            n = int(user_input)
        except ValueError:
            utils.print("Error: Invalid input. Please enter a number.")
            return edges["onError"], state

        return edges["onSuccess"], {**state, "count": n}

    return node
