"""The counter walk: see ``specs/counter.md``."""

import asyncio
import sys
from dataclasses import dataclass, field
from enum import Enum, auto

from {{package}}.engine import Walk
from {{package}}.nodes import count, start
from {{package}}.nodes.count import Count
from {{package}}.nodes.start import Start
from {{package}}.nodes.stop import Stop

__all__ = ["Error", "main", "run"]

Error = start.Error
"""What can stop the walk early."""


@dataclass
class State:
    count: int = 0


class Id(Enum):
    START = auto()
    COUNT = auto()
    STOP = auto()


@dataclass(frozen=True)
class Counter(Walk[Id, State]):
    start: Start = field(default_factory=Start)
    count: Count = field(default_factory=Count)
    stop: Stop = field(default_factory=Stop)

    async def step(self, id: Id, state: State) -> Id | None:
        match id:
            case Id.START:
                match await self.start.run(state):
                    case start.Outcome.SUCCESS:
                        return Id.COUNT
                    case start.Outcome.INVALID_INPUT:
                        return Id.START
            case Id.COUNT:
                match await self.count.run(state):
                    case count.Outcome.AGAIN:
                        return Id.COUNT
                    case count.Outcome.FINISHED:
                        return Id.STOP
            case Id.STOP:
                await self.stop.run(state)
                return None


async def run() -> None:
    """Runs the walk from its first node."""
    await Counter().run(Id.START, State())


def main() -> None:
    """Runs the walk and reports an error: the ``counter`` script in pyproject.toml."""
    try:
        asyncio.run(run())
    except Error as e:
        sys.exit(f"Error: {e}")
