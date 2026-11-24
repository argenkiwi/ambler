from dataclasses import dataclass, field
from typing import Protocol

from {{package}}.nodes import HasCount


class Utils(Protocol):
    def print(self, message: str) -> None: ...


class SystemUtils:
    def print(self, message: str) -> None:
        print(message)


@dataclass(frozen=True)
class Stop:
    """Prints the final count. Terminal: it has no outgoing edges."""

    utils: Utils = field(default_factory=SystemUtils)

    async def run(self, state: HasCount) -> None:
        self.utils.print(f"Final count: {state.count}")
