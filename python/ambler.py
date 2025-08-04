from typing import TypeVar, Tuple, Callable, Optional, Awaitable

S = TypeVar('S')
L = TypeVar('L')


async def amble(state: S, lead: L, follow: Callable[[L], Callable[[S], Awaitable[Tuple[S, Optional[L]]]]]) -> S:
    resolve = follow(lead)
    current_state, next_lead = await resolve(state)
    if next_lead is None:
        return current_state
    else:
        return await amble(current_state, next_lead, follow)
