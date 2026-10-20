import asyncio
import importlib
import inspect
import sys
import os
from typing import Literal, Optional, TypedDict

# Add parent directory to path to import ambler and nodes
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ambler import ambler


class State(TypedDict):
    count: int


NodeId = Literal["START", "COUNT", "STOP"]

# Node modules are imported lazily, the first time each node runs.
amble = ambler({
    "START": lambda: importlib.import_module("nodes.start").factory({"onSuccess": "COUNT", "onError": "START"}),
    "COUNT": lambda: importlib.import_module("nodes.count").factory({"onCount": "COUNT", "onStop": "STOP"}),
    "STOP": lambda: importlib.import_module("nodes.stop").factory({"onDone": None}),
})


async def main():
    node_id: Optional[NodeId] = "START"
    state: State = {"count": 0}

    while node_id:
        next = amble(node_id, state)
        node_id, state = await next if inspect.isawaitable(next) else next


if __name__ == "__main__":
    asyncio.run(main())
