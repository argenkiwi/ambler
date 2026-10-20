from typing import Dict, Any, Protocol
from ambler import Edges, Node, Next

class Utils(Protocol):
    def log(self, message: str) -> None:
        ...

class DefaultUtils:
    def log(self, message: str) -> None:
        print(message)

def create(
    edges: Edges[str, str],
    utils: Utils = DefaultUtils()
) -> Node[Dict[str, Any], str]:
    def node_fn(state: Dict[str, Any], key: str) -> Next[Dict[str, Any], str]:
        utils.log(f"Final count: {state['count']}")
        utils.log("Counting process terminated.")
        return edges["onDone"], state

    return node_fn
