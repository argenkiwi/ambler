from typing import Tuple, Optional

from lead import Lead


def start(state: int) -> Tuple[int, Optional[Lead]]:
    user_input = input("Enter a starting number (or press Enter for default 0): ")
    if not user_input:
        print(f"Starting count from default: {state}")
        return state, Lead.COUNT
    try:
        initial_count = int(user_input)
        state = initial_count
        print(f"Starting count from: {state}")
        return state, Lead.COUNT
    except ValueError:
        print("Invalid input. Please, try again.")
        return state, Lead.START
