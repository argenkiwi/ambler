from dataclasses import dataclass, field

from {{package}}.nodes.stop import Stop


@dataclass
class MockState:
    count: int


@dataclass
class MockUtils:
    printed: list[str] = field(default_factory=list[str])

    def print(self, message: str) -> None:
        self.printed.append(message)


async def test_prints_final_count_and_leaves_state_untouched() -> None:
    utils = MockUtils()
    state = MockState(15)

    await Stop(utils).run(state)

    assert state.count == 15
    assert utils.printed == ["Final count: 15"]
