from abc import ABC, abstractmethod
from typing import TypeVar, Generic, Tuple, Optional

# Define a generic type 'S' for the state, similar to Kotlin/TypeScript generics.
S = TypeVar("S")


# An abstract base class defining the interface for a step in the state machine.
# This is the Python equivalent of the `sealed interface`.
class Step(Generic[S], ABC):
    @abstractmethod
    async def run(self, state: S) -> Tuple[S, Optional["Step[S]"]]:
        """
        Runs a step, returning the new state and the next step.
        A 'None' next step terminates the process.
        """
        pass


# --- Executor Function ---

async def amble(initial_state: S, initial_step: Step[S]) -> S:
    """
    Executes the state machine using a `while` loop. This is the idiomatic
    Python equivalent of the original's tail-recursive function.
    """
    current_state = initial_state
    current_step: Optional[Step[S]] = initial_step

    while current_step is not None:
        # Await the result of the current step's execution.
        new_state, next_step = await current_step.run(current_state)
        current_state = new_state
        current_step = next_step

    return current_state
