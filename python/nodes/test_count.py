import unittest
from nodes.count import factory


class MockUtils:
    def __init__(self, random):
        self._random = random

    def print(self, message):
        pass

    async def sleep(self, ms):
        pass

    def random(self):
        return self._random


class TestCountNode(unittest.IsolatedAsyncioTestCase):
    async def test_increments_count_and_transitions_to_on_count_if_random_above_half(self):
        node = factory({"onCount": "onCount", "onStop": "onStop"}, MockUtils(0.6))
        next_id, next_state = await node({"count": 5})

        self.assertEqual(next_id, "onCount")
        self.assertEqual(next_state["count"], 6)

    async def test_increments_count_and_transitions_to_on_stop_if_random_at_most_half(self):
        node = factory({"onCount": "onCount", "onStop": "onStop"}, MockUtils(0.4))
        next_id, next_state = await node({"count": 10})

        self.assertEqual(next_id, "onStop")
        self.assertEqual(next_state["count"], 11)


if __name__ == "__main__":
    unittest.main()
