from typing import Tuple, Optional

from lead import Lead


async def stop(state: int) -> Tuple[int, Optional[Lead]]:
    print(f"Final count: {state}")
    return state, None
