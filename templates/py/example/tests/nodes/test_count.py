from dataclasses import dataclass

from {{package}}.nodes.count import Count, Outcome


@dataclass
class MockState:
    count: int


@dataclass
class MockUtils:
    heads: bool

    def print(self, message: str) -> None:
        pass

    async def sleep(self, seconds: float) -> None:
        pass

    def random_bool(self, p: float) -> bool:
        return self.heads


async def run(heads: bool, count: int) -> tuple[Outcome, int]:
    state = MockState(count)
    outcome = await Count(MockUtils(heads)).run(state)
    return outcome, state.count


async def test_increments_count_and_goes_again_if_coin_lands_heads() -> None:
    assert await run(True, 5) == (Outcome.AGAIN, 6)


async def test_increments_count_and_finishes_if_coin_lands_tails() -> None:
    assert await run(False, 10) == (Outcome.FINISHED, 11)
