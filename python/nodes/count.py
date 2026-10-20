import asyncio
import random
from typing import Awaitable, Literal, Optional, Protocol, TypedDict, TypeVar
from ambler import Edges, Node

K = TypeVar("K", bound=str)


class State(TypedDict):
    count: int


Edge = Literal["onCount", "onStop"]


class Utils(Protocol):
    def print(self, message: str) -> None: ...
    def sleep(self, ms: int) -> Awaitable[None]: ...
    def random(self) -> float: ...


class DefaultUtils:
    def print(self, message: str) -> None:
        print(message)

    async def sleep(self, ms: int) -> None:
        await asyncio.sleep(ms / 1000)

    def random(self) -> float:
        return random.random()


def factory(edges: Edges[Edge, K], utils: Optional[Utils] = None) -> Node[State, K]:
    utils = utils or DefaultUtils()

    async def node(state: State):
        utils.print(f"Current count: {state['count']}")
        await utils.sleep(1000)
        next_state = {**state, "count": state["count"] + 1}

        if utils.random() > 0.5:
            return edges["onCount"], next_state
        else:
            return edges["onStop"], next_state

    return node
