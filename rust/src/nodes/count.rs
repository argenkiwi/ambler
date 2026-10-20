use crate::{Next, Node};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;
use crate::nodes::start_node::HasCount;

#[async_trait]
pub trait Utils: Send + Sync {
    fn log(&self, message: &str);
    async fn sleep(&self, duration: Duration);
    fn random(&self) -> f64;
}

pub struct DefaultUtils;

#[async_trait]
impl Utils for DefaultUtils {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
    fn random(&self) -> f64 {
        rand::random::<f64>()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Edge {
    OnCount,
    OnStop,
}

pub struct CountNode<K> {
    pub edges: HashMap<Edge, Option<K>>,
    pub utils: Box<dyn Utils>,
}

impl<K> CountNode<K> {
    pub fn new(edges: HashMap<Edge, Option<K>>, utils: Box<dyn Utils>) -> Self {
        Self { edges, utils }
    }
}

#[async_trait]
impl<S, K> Node<S, K> for CountNode<K>
where
    S: HasCount + Send + Sync + 'static,
    K: Clone + Send + Sync + std::hash::Hash + std::cmp::Eq + 'static,
{
    async fn run(&self, mut state: S, _key: K) -> Next<S, K> {
        self.utils.log(&format!("Current count: {}", state.count()));

        self.utils.sleep(Duration::from_secs(1)).await;

        state.set_count(state.count() + 1);

        let stop = self.utils.random() > 0.7;

        let next_node = if stop {
            self.edges.get(&Edge::OnStop).cloned().flatten()
        } else {
            self.edges.get(&Edge::OnCount).cloned().flatten()
        };

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
        fn set_count(&mut self, count: i32) {
            self.count = count;
        }
    }

    struct MockUtils {
        random_val: f64,
    }
    #[async_trait]
    impl Utils for MockUtils {
        fn log(&self, _message: &str) {}
        async fn sleep(&self, _duration: Duration) {}
        fn random(&self) -> f64 {
            self.random_val
        }
    }

    #[tokio::test]
    async fn test_count_node_continue() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnCount, Some("count".to_string()));

        let node = CountNode::new(edges, Box::new(MockUtils { random_val: 0.5 }));
        let state = MockState { count: 5 };

        let (next, next_state) = node.run(state, "count".to_string()).await;

        assert_eq!(next, Some("count".to_string()));
        assert_eq!(next_state.count, 6);
    }

    #[tokio::test]
    async fn test_count_node_stop() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnStop, Some("stop".to_string()));

        let node = CountNode::new(edges, Box::new(MockUtils { random_val: 0.8 }));
        let state = MockState { count: 5 };

        let (next, next_state) = node.run(state, "count".to_string()).await;

        assert_eq!(next, Some("stop".to_string()));
        assert_eq!(next_state.count, 6);
    }
}
