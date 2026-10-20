import unittest
from nodes.start import factory


class MockUtils:
    def __init__(self, input):
        self.input = input

    def read_line(self, message):
        return self.input

    def print(self, message):
        pass


class TestStartNode(unittest.TestCase):
    def test_transitions_to_on_success_with_0_if_input_is_empty(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils(""))
        next_id, next_state = node({"count": 0})

        self.assertEqual(next_id, "NEXT")
        self.assertEqual(next_state["count"], 0)

    def test_transitions_to_on_success_with_input_number(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils("42"))
        next_id, next_state = node({"count": 0})

        self.assertEqual(next_id, "NEXT")
        self.assertEqual(next_state["count"], 42)

    def test_transitions_to_on_error_if_input_is_invalid(self):
        node = factory({"onSuccess": "NEXT", "onError": "ERROR"}, MockUtils("abc"))
        next_id, next_state = node({"count": 123})

        self.assertEqual(next_id, "ERROR")
        self.assertEqual(next_state["count"], 123)


if __name__ == "__main__":
    unittest.main()
