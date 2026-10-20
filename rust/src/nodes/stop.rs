use crate::{Next, Node};
use async_trait::async_trait;
use std::collections::HashMap;
use crate::nodes::start_node::HasCount;

pub trait Utils: Send + Sync {
    fn log(&self, message: &str);
}

pub struct DefaultUtils;
impl Utils for DefaultUtils {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Edge {
    OnDone,
}

pub struct StopNode<K> {
    pub edges: HashMap<Edge, Option<K>>,
    pub utils: Box<dyn Utils>,
}

impl<K> StopNode<K> {
    pub fn new(edges: HashMap<Edge, Option<K>>, utils: Box<dyn Utils>) -> Self {
        Self { edges, utils }
    }
}

#[async_trait]
impl<S, K> Node<S, K> for StopNode<K>
where
    S: HasCount + Send + Sync + 'static,
    K: Clone + Send + Sync + std::hash::Hash + std::cmp::Eq + 'static,
{
    async fn run(&self, state: S, _key: K) -> Next<S, K> {
        self.utils.log(&format!("Final count: {}", state.count()));
        self.utils.log("Counting process terminated.");

        let next_node = self.edges.get(&Edge::OnDone).cloned().flatten();

        (next_node, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockState {
        count: i32,
    }
    impl HasCount for MockState {
        fn count(&self) -> i32 {
            self.count
        }
        fn set_count(&mut self, _count: i32) {}
    }

    struct MockUtils;
    impl Utils for MockUtils {
        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_stop_node() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnDone, None);

        let node = StopNode::new(edges, Box::new(MockUtils));
        let state = MockState { count: 10 };

        let (next, next_state) = node.run(state, "stop".to_string()).await;

        assert_eq!(next, None);
        assert_eq!(next_state.count, 10);
    }
}
