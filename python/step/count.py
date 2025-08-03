import asyncio
import random
from typing import Tuple, Optional

from lead import Lead


async def count(state: int) -> Tuple[int, Optional[Lead]]:
    print(f"Current count: {state}")
    await asyncio.sleep(1)
    state += 1
    if random.choice([True, False]):
        return state, Lead.COUNT
    else:
        return state, Lead.STOP
