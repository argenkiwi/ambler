import unittest
from stop_node import create

class TestStopNode(unittest.TestCase):
    def test_stop_node_done(self):
        state = {"count": 10}
        class MockUtils:
            def log(self, message): pass
        
        node = create({"onDone": None}, MockUtils())
        next_id, next_state = node(state, "stop")
        
        self.assertEqual(next_id, None)
        self.assertEqual(next_state["count"], 10)

if __name__ == "__main__":
    unittest.main()
