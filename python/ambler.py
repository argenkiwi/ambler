from typing import TypeVar, Tuple, Callable, Optional, Awaitable

S = TypeVar('S')
L = TypeVar('L')


async def amble(state: S, lead: L, follow: Callable[[L], Callable[[S], Awaitable[Tuple[S, Optional[L]]]]]) -> S:
    current_state, next_lead = await follow(lead)(state)
    if next_lead is None:
        return current_state
    else:
        return await amble(current_state, next_lead, follow)
