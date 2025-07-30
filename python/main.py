import asyncio
import random
from typing import Optional

from ambler import Next, amble_from

def prompt_for_number() -> int:
    while True:
        try:
            return int(input("Enter a starting number: "))
        except ValueError:
            print("Invalid number, please try again.")

async def prompt_number_node(state: int) -> Optional[Next]:
    number = prompt_for_number()
    return Next(lambda: start_node(number))

async def start_node(state: int) -> Optional[Next]:
    print(f"Starting count from {state}")
    return Next(lambda: step_node(state))

async def step_node(state: int) -> Optional[Next]:
    new_state = state + 1
    print(f"Count: {new_state}")
    if random.random() > 0.5:
        return Next(lambda: step_node(new_state))
    else:
        return Next(lambda: stop_node(new_state))

async def stop_node(state: int) -> Optional[Next]:
    print("Stopping count.")
    return None

if __name__ == "__main__":
    asyncio.run(amble_from(lambda: prompt_number_node(0)))
