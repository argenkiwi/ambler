from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Protocol

from {{package}}.nodes import HasCount
from {{package}}.utils import terminal


class Outcome(Enum):
    """Where :class:`Start` leaves the walk."""

    SUCCESS = auto()
    """The count was set, from the input or to 0 if there was none."""
    INVALID_INPUT = auto()
    """The input was not a number; the count is untouched."""


class Error(Exception):
    """The starting number could not be read."""


class Utils(Protocol):
    def read_line(self, message: str) -> str | None:
        """Prints ``message`` and reads a line, or ``None`` at end of input."""
        ...

    def print(self, message: str) -> None: ...


class SystemUtils:
    def read_line(self, message: str) -> str | None:
        return terminal.prompt(message)

    def print(self, message: str) -> None:
        print(message)


@dataclass(frozen=True)
class Start:
    """Asks for a starting number. Empty input or end of input counts as 0."""

    utils: Utils = field(default_factory=SystemUtils)

    async def run(self, state: HasCount) -> Outcome:
        try:
            text = self.utils.read_line("Enter a starting number: ")
        except OSError as e:
            raise Error("could not read the starting number") from e

        if not text:
            state.count = 0
            return Outcome.SUCCESS
        try:
            state.count = int(text)
        except ValueError:
            self.utils.print("Error: Invalid input. Please enter a number.")
            return Outcome.INVALID_INPUT
        return Outcome.SUCCESS
