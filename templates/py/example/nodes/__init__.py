"""Reusable atomic step nodes and the shared state protocols they require."""

from typing import Protocol


class HasCount(Protocol):
    """A state that has a count field."""

    count: int
