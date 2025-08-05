from typing import TypeVar, Tuple, Callable, Optional, Awaitable

S = TypeVar('S')
L = TypeVar('L')


async def amble(state: S, lead: L, follow: Callable[[L, S], Awaitable[Tuple[S, Optional[L]]]]) -> S:
    current_state = state
    current_lead = lead
    while current_lead is not None:
        current_state, current_lead = await follow(current_lead, current_state)
    return current_state
