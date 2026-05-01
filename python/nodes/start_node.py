from typing import Dict, Any, Optional, Protocol, Union, TypeVar
from ambler import Edges, Node, Next

class Utils(Protocol):
    def prompt(self, message: str) -> Optional[str]:
        ...
    def error(self, message: str) -> None:
        ...

class DefaultUtils:
    def prompt(self, message: str) -> Optional[str]:
        try:
            return input(message)
        except EOFError:
            return None

    def error(self, message: str) -> None:
        print(message)

Edge = Union[str, str]  # "onSuccess" | "onError"

def create(
    edges: Edges[str, str],
    utils: Utils = DefaultUtils()
) -> Node[Dict[str, Any], str]:
    def node_fn(state: Dict[str, Any], key: str) -> Next[Dict[str, Any], str]:
        user_input = utils.prompt("Enter a starting number (default 0): ")

        if user_input is None or user_input.strip() == "":
            return edges["onSuccess"], {**state, "count": 0}

        try:
            count = int(user_input)
        except ValueError:
            utils.error("Invalid number entered.")
            return edges["onError"], state

        return edges["onSuccess"], {**state, "count": count}

    return node_fn
