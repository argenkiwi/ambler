import unittest
from count_node import create

class TestCountNode(unittest.IsolatedAsyncioTestCase):
    async def test_count_node_increment_on_count(self):
        state = {"count": 5}
        class MockUtils:
            def log(self, message): pass
            async def sleep(self, ms): pass
            def random(self): return 0.5
        
        node = create({"onCount": "count", "onStop": "stop"}, MockUtils())
        next_id, next_state = await node(state, "count")
        
        self.assertEqual(next_id, "count")
        self.assertEqual(next_state["count"], 6)

    async def test_count_node_increment_on_stop(self):
        state = {"count": 5}
        class MockUtils:
            def log(self, message): pass
            async def sleep(self, ms): pass
            def random(self): return 0.8
        
        node = create({"onCount": "count", "onStop": "stop"}, MockUtils())
        next_id, next_state = await node(state, "count")
        
        self.assertEqual(next_id, "stop")
        self.assertEqual(next_state["count"], 6)

if __name__ == "__main__":
    unittest.main()
