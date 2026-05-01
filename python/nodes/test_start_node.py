import unittest
from start_node import create

class TestStartNode(unittest.TestCase):
    def test_start_node_success_empty_input(self):
        state = {"count": 0}
        class MockUtils:
            def prompt(self, message): return ""
            def error(self, message): pass
        
        node = create({"onSuccess": "count", "onError": "start"}, MockUtils())
        next_id, next_state = node(state, "start")
        
        self.assertEqual(next_id, "count")
        self.assertEqual(next_state["count"], 0)

    def test_start_node_success_valid_input(self):
        state = {"count": 0}
        class MockUtils:
            def prompt(self, message): return "42"
            def error(self, message): pass
        
        node = create({"onSuccess": "count", "onError": "start"}, MockUtils())
        next_id, next_state = node(state, "start")
        
        self.assertEqual(next_id, "count")
        self.assertEqual(next_state["count"], 42)

    def test_start_node_error_invalid_input(self):
        state = {"count": 10}
        class MockUtils:
            def prompt(self, message): return "invalid"
            def error(self, message): pass
        
        node = create({"onSuccess": "count", "onError": "start"}, MockUtils())
        next_id, next_state = node(state, "start")
        
        self.assertEqual(next_id, "start")
        self.assertEqual(next_state["count"], 10)

if __name__ == "__main__":
    unittest.main()
