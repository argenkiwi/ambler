from typing import Optional, Callable, Awaitable

class Next:
    def __init__(self, run: Callable[[], Awaitable[Optional['Next']]]):
        self.run = run

async def amble(initial: Optional[Next]):
    next_step = initial
    while next_step is not None:
        next_step = await next_step.run()

async def amble_from(initial: Callable[[], Awaitable[Optional['Next']]]):
    await amble(Next(initial))
