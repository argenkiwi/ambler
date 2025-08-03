from typing import Tuple, Optional

from lead import Lead


async def start(state: int) -> Tuple[int, Optional[Lead]]:
    while True:
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
            print("Invalid input. Please enter a valid number or press Enter.")
