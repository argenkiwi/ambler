import asyncio
import random
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Protocol

from {{package}}.nodes import HasCount


class Outcome(Enum):
    """Where :class:`Count` leaves the walk."""

    AGAIN = auto()
    """Keep counting."""
    FINISHED = auto()
    """Stop counting."""


class Utils(Protocol):
    def print(self, message: str) -> None: ...

    async def sleep(self, seconds: float) -> None: ...

    def random_bool(self, p: float) -> bool:
        """Returns ``True`` with probability ``p``."""
        ...


class SystemUtils:
    def print(self, message: str) -> None:
        print(message)

    async def sleep(self, seconds: float) -> None:
        await asyncio.sleep(seconds)

    def random_bool(self, p: float) -> bool:
        return random.random() < p


@dataclass(frozen=True)
class Count:
    """Prints the count, waits a second, increments it, then flips a coin between
    :attr:`Outcome.AGAIN` and :attr:`Outcome.FINISHED`."""

    utils: Utils = field(default_factory=SystemUtils)

    async def run(self, state: HasCount) -> Outcome:
        self.utils.print(f"Current count: {state.count}")
        await self.utils.sleep(1)
        state.count += 1

        return Outcome.AGAIN if self.utils.random_bool(0.5) else Outcome.FINISHED
