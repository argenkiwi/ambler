import asyncio
import inspect
import sys
import os

# Add parent directory to path to import ambler and nodes
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from ambler import ambler
from nodes import start_node, count_node, stop_node

async def main():
    nodes = {
        "start": start_node.create({"onSuccess": "count", "onError": "start"}),
        "count": count_node.create({"onCount": "count", "onStop": "stop"}),
        "stop": stop_node.create({"onDone": None}),
    }

    amble = ambler(nodes)

    node_id = "start"
    state = {"count": 0}

    while node_id:
        result = amble(node_id, state)
        if inspect.isawaitable(result):
            node_id, state = await result
        else:
            node_id, state = result

if __name__ == "__main__":
    asyncio.run(main())
