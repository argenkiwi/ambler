import time
import random
from typing import Optional

from ambler import Next, amble

def prompt_for_number() -> int:
    while True:
        try:
            return int(input("Enter a starting number: "))
        except ValueError:
            print("Invalid number, please try again.")

def prompt_number_node() -> Next:
    number = prompt_for_number()
    return Next(step_node, number)

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
    amble(prompt_number_node())
