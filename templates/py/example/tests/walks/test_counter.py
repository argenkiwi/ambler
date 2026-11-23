from collections import deque
from collections.abc import Iterable

from {{package}}.nodes.count import Count
from {{package}}.nodes.start import Start
from {{package}}.nodes.stop import Stop
from {{package}}.walks.counter import Counter, Id, State


class Script:
    """Plays back typed lines and coin flips, and records everything printed.

    Implements the ``Utils`` protocol of every node in the walk.
    """

    def __init__(self, lines: Iterable[str], flips: Iterable[bool]) -> None:
        self.lines = deque(lines)
        self.flips = deque(flips)
        self.printed: list[str] = []

    def read_line(self, message: str) -> str | None:
        return self.lines.popleft() if self.lines else None

    def print(self, message: str) -> None:
        self.printed.append(message)

    async def sleep(self, seconds: float) -> None:
        pass

    def random_bool(self, p: float) -> bool:
        return self.flips.popleft()


def counter(script: Script) -> Counter:
    return Counter(start=Start(script), count=Count(script), stop=Stop(script))


async def test_retries_invalid_input_then_counts_until_the_coin_says_stop() -> None:
    script = Script(["abc", "3"], [True, False])
    state = State(count=99)

    await counter(script).run(Id.START, state)

    assert state.count == 5
    assert script.printed == [
        "Error: Invalid input. Please enter a number.",
        "Current count: 3",
        "Current count: 4",
        "Final count: 5",
    ]
