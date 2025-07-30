import random
import time
from typing import Optional

from ambler import Next, amble


def prompt_number_node(state: int) -> Next:
    try:
        text = input("Enter a starting number (defaults to 0): ")
        if len(text) > 0:
            number = int(text)
        else:
            number = state

        return Next(step_node, number)
    except ValueError:
        print("Invalid number.")
        return Next(prompt_number_node, state)


def step_node(state: int) -> Optional[Next]:
    print(f"Count: {state}")
    time.sleep(1)
    new_state = state + 1
    if random.random() > 0.5:
        return Next(step_node, new_state)
    else:
        return Next(stop_node, new_state)


def stop_node(state: int) -> Optional[Next]:
    print(f"Stopping count at {state}.")
    return None


if __name__ == "__main__":
    amble(prompt_number_node(0))
