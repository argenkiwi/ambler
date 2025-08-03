import asyncio
import random
from enum import Enum, auto
from typing import Tuple, Optional

from ambler import amble, Next


class CounterLead(Enum):
    START = auto()
    COUNT = auto()
    STOP = auto()


async def start(state: int) -> Tuple[int, Optional[CounterLead]]:
    while True:
        user_input = input("Enter a starting number (or press Enter for default 0): ")
        if not user_input:
            print(f"Starting count from default: {state}")
            return state, CounterLead.COUNT
        try:
            initial_count = int(user_input)
            state = initial_count
            print(f"Starting count from: {state}")
            return state, CounterLead.COUNT
        except ValueError:
            print("Invalid input. Please enter a valid number or press Enter.")


async def count(state: int) -> Tuple[int, Optional[CounterLead]]:
    print(f"Current count: {state}")
    await asyncio.sleep(1)
    state += 1
    if random.choice([True, False]):
        return state, CounterLead.COUNT
    else:
        return state, CounterLead.STOP


async def stop(state: int) -> Tuple[int, Optional[CounterLead]]:
    print(f"Final count: {state}")
    return state, None


async def main():
    initial_state = 0
    initial_lead = CounterLead.START

    def follow(lead: CounterLead):
        match lead:
            case CounterLead.START:
                return Next(start)
            case CounterLead.COUNT:
                return Next(count)
            case CounterLead.STOP:
                return Next(stop)
            case _:
                raise ValueError(f"Unknown lead: {lead}")

    await amble(initial_state, initial_lead, follow)


if __name__ == "__main__":
    asyncio.run(main())
