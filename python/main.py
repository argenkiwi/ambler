import random
import time
from typing import Optional

from ambler import Next, amble


def def start(state: int) -> Next:
    text = input("Enter a starting number (or press Enter for default): ")
    if not text:
        print("Using default starting number.")
        return Next(count, state)
    try:
        return Next(count, int(text))
    except ValueError:
        print("Invalid number, please try again.")
        return Next(start, state)


def count(state: int) -> Optional[Next]:
    print(f"Count: {state}")
    time.sleep(1)
    new_state = state + 1
    if random.random() > 0.5:
        return Next(count, new_state)
    else:
        return Next(stop, new_state)


def stop(state: int) -> Optional[Next]:
    print(f"Stopping count at {state}.")
    return None


if __name__ == "__main__":
    amble(start, 0)
