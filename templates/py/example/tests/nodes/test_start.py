from dataclasses import dataclass

import pytest

from {{package}}.nodes.start import Error, Outcome, Start


@dataclass
class MockState:
    count: int


@dataclass
class MockUtils:
    input: str | OSError | None

    def read_line(self, message: str) -> str | None:
        if isinstance(self.input, OSError):
            raise self.input
        return self.input

    def print(self, message: str) -> None:
        pass


async def run(input: str | None, count: int) -> tuple[Outcome, int]:
    state = MockState(count)
    outcome = await Start(MockUtils(input)).run(state)
    return outcome, state.count


async def test_succeeds_with_0_if_input_is_empty() -> None:
    assert await run("", 7) == (Outcome.SUCCESS, 0)


async def test_succeeds_with_0_at_end_of_input() -> None:
    assert await run(None, 7) == (Outcome.SUCCESS, 0)


async def test_succeeds_with_input_number() -> None:
    assert await run("42", 0) == (Outcome.SUCCESS, 42)


async def test_reports_invalid_input_and_keeps_count() -> None:
    assert await run("abc", 123) == (Outcome.INVALID_INPUT, 123)


async def test_fails_if_input_cannot_be_read() -> None:
    state = MockState(1)

    with pytest.raises(Error):
        await Start(MockUtils(BrokenPipeError())).run(state)

    assert state.count == 1
