from abc import ABC, abstractmethod
from typing import TypeVar, Tuple, Callable, Optional, Awaitable

S = TypeVar('S')
L = TypeVar('L')

class Step(ABC):
    @abstractmethod
    async def resolve(self, state: S) -> Tuple[S, Optional[L]]:
        pass

class Next(Step):
    def __init__(self, delegate: Callable[[S], Awaitable[Tuple[S, Optional[L]]]]):
        self.delegate = delegate

    async def resolve(self, state: S) -> Tuple[S, Optional[L]]:
        return await self.delegate(state)

async def amble(state: S, lead: L, follow: Callable[[L], Step]) -> S:
    current_state, next_lead = await follow(lead).resolve(state)
    if next_lead is None:
        return current_state
    else:
        return await amble(current_state, next_lead, follow)