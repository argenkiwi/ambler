"""A predictable architecture for writing programs as graphs.

A program is a set of small nodes joined into a walk. Each node does one thing to
shared state and reports what happened; the walk decides where to go next.
"""

from abc import ABC, abstractmethod
from typing import Protocol

__all__ = ["Node", "Walk"]


class Node[S, O](Protocol):
    """A processing step in a walk.

    A node reads and mutates the shared state, then reports what happened as an
    outcome (usually a member of the node's own ``Outcome`` enum, or ``None`` for a
    terminal node). It never names the node that comes next: wiring outcomes to
    successors is the walk's job (see :class:`Walk`).

    A node that cannot complete raises its module's ``Error``; the walk decides
    what that means.
    """

    async def run(self, state: S, /) -> O: ...


class Walk[Id, S](ABC):
    """A graph of nodes, wired together.

    A subclass owns its nodes and matches on an ``Id`` enum in :meth:`step`, turning
    each node's outcome into the next id, or ``None`` to halt. The matches have no
    wildcard arm, so pyright (``reportMatchNotExhaustive``) reports an unwired
    outcome or an unknown node as an error.
    """

    @abstractmethod
    async def step(self, id: Id, state: S) -> Id | None:
        """Runs the node ``id`` and returns the next id, or ``None`` to halt."""

    async def run(self, start: Id, state: S) -> None:
        """Steps from ``start`` until a node halts the walk.

        Any exception raised by :meth:`step` stops the walk and propagates.
        """
        id: Id | None = start
        while id is not None:
            id = await self.step(id, state)
