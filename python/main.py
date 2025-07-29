import asyncio
import random
from typing import Tuple, Optional

from ambler import amble, Step


# --- State Machine Step Definitions ---

class Start(Step[int]):
    async def run(self, state: int) -> Tuple[int, Optional[Step[int]]]:
        print(f"Starting from {state}...")
        # Note: We return the singleton instance `Count_step` here.
        return state, Count_step


class Count(Step[int]):
    async def run(self, state: int) -> Tuple[int, Optional[Step[int]]]:
        count = state + 1
        await asyncio.sleep(1)  # Equivalent to Kotlin's `delay`.
        print(f"...{count}...")
        # Randomly choose the next step.
        next_step = Count_step if random.choice([True, False]) else Stop_step
        return count, next_step


class Stop(Step[int]):
    async def run(self, state: int) -> Tuple[int, Optional[Step[int]]]:
        print("...and stop.")
        # Return `None` as the next step to terminate the loop.
        return state, None


# Create singleton instances of each step. This is the equivalent of `data object`.
Start_step = Start()
Count_step = Count()
Stop_step = Stop()


# --- Main Execution ---

async def main():
    """The main asynchronous entry point for the program."""
    await amble(0, Start_step)


if __name__ == "__main__":
    # `asyncio.run()` is the standard way to run a top-level async function.
    # It manages the event loop and is functionally similar to `runBlocking`.
    asyncio.run(main())
