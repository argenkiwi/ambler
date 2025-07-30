from typing import Any, Callable, Optional

class Next:
    def __init__(self, next_func: Callable[..., Any], state: Any):
        self.next_func = next_func
        self.state = state

    def __call__(self) -> Optional['Next']:
        return self.next_func(self.state)

def amble(initial: Next):
    next_step = initial
    while next_step is not None:
        next_step = next_step()
