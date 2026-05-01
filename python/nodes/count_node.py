import asyncio
import random
from typing import Dict, Any, Protocol, Union, Awaitable
from ambler import Edges, Node, Next

class Utils(Protocol):
    def log(self, message: str) -> None:
        ...
    def sleep(self, ms: int) -> Awaitable[None]:
        ...
    def random(self) -> float:
        ...

class DefaultUtils:
    def log(self, message: str) -> None:
        print(message)
    
    async def sleep(self, ms: int) -> None:
        await asyncio.sleep(ms / 1000)

    def random(self) -> float:
        return random.random()

Edge = Union[str, str] # "onCount" | "onStop"

def create(
    edges: Edges[str, str],
    utils: Utils = DefaultUtils()
) -> Node[Dict[str, Any], str]:
    async def node_fn(state: Dict[str, Any], key: str) -> Next[Dict[str, Any], str]:
        utils.log(f"Current count: {state['count']}")

        await utils.sleep(1000)

        next_state = {**state, "count": state["count"] + 1}

        stop = utils.random() > 0.7

        if stop:
            return edges["onStop"], next_state

        return edges["onCount"], next_state

    return node_fn
