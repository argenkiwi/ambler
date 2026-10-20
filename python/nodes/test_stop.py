import unittest
from nodes.stop import factory


class MockUtils:
    def __init__(self):
        self.message = None

    def print(self, message):
        self.message = message


class TestStopNode(unittest.TestCase):
    def test_prints_final_count_and_transitions_to_on_done(self):
        utils = MockUtils()
        node = factory({"onDone": None}, utils)
        next_id, next_state = node({"count": 15})

        self.assertIsNone(next_id)
        self.assertEqual(next_state["count"], 15)
        self.assertEqual(utils.message, "Final count: 15")


if __name__ == "__main__":
    unittest.main()
