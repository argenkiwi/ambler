use crate::{Next, Node};
use async_trait::async_trait;
use std::collections::HashMap;

pub trait Utils: Send + Sync {
    fn prompt(&self, message: &str) -> Option<String>;
    fn error(&self, message: &str);
}

pub struct DefaultUtils;
impl Utils for DefaultUtils {
    fn prompt(&self, message: &str) -> Option<String> {
        use std::io::{self, Write};
        print!("{}", message);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        Some(input.trim().to_string())
    }
    fn error(&self, message: &str) {
        eprintln!("{}", message);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Edge {
    OnSuccess,
    OnError,
}

pub trait HasCount {
    fn count(&self) -> i32;
    fn set_count(&mut self, count: i32);
}

pub struct StartNode<K> {
    pub edges: HashMap<Edge, Option<K>>,
    pub utils: Box<dyn Utils>,
}

impl<K> StartNode<K> {
    pub fn new(edges: HashMap<Edge, Option<K>>, utils: Box<dyn Utils>) -> Self {
        Self { edges, utils }
    }
}

#[async_trait]
impl<S, K> Node<S, K> for StartNode<K>
where
    S: HasCount + Send + Sync + 'static,
    K: Clone + Send + Sync + std::hash::Hash + std::cmp::Eq + 'static,
{
    async fn run(&self, mut state: S, _key: K) -> Next<S, K> {
        let input = self.utils.prompt("Enter a starting number (default 0):");

        let next_node = match input {
            Some(s) if s.is_empty() => {
                state.set_count(0);
                self.edges.get(&Edge::OnSuccess).cloned().flatten()
            }
            Some(s) => match s.parse::<i32>() {
                Ok(count) => {
                    state.set_count(count);
                    self.edges.get(&Edge::OnSuccess).cloned().flatten()
                }
                Err(_) => {
                    self.utils.error("Invalid number entered.");
                    self.edges.get(&Edge::OnError).cloned().flatten()
                }
            },
            None => {
                state.set_count(0);
                self.edges.get(&Edge::OnSuccess).cloned().flatten()
            }
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
        input: Option<String>,
    }
    impl Utils for MockUtils {
        fn prompt(&self, _message: &str) -> Option<String> {
            self.input.clone()
        }
        fn error(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_start_node_empty_input() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnSuccess, Some("count".to_string()));

        let node = StartNode::new(edges, Box::new(MockUtils { input: Some("".to_string()) }));
        let state = MockState { count: 10 };

        let (next, next_state) = node.run(state, "start".to_string()).await;

        assert_eq!(next, Some("count".to_string()));
        assert_eq!(next_state.count, 0);
    }

    #[tokio::test]
    async fn test_start_node_valid_input() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnSuccess, Some("count".to_string()));

        let node = StartNode::new(edges, Box::new(MockUtils { input: Some("42".to_string()) }));
        let state = MockState { count: 10 };

        let (next, next_state) = node.run(state, "start".to_string()).await;

        assert_eq!(next, Some("count".to_string()));
        assert_eq!(next_state.count, 42);
    }

    #[tokio::test]
    async fn test_start_node_invalid_input() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnError, Some("start".to_string()));

        let node = StartNode::new(edges, Box::new(MockUtils { input: Some("invalid".to_string()) }));
        let state = MockState { count: 10 };

        let (next, next_state) = node.run(state, "start".to_string()).await;

        assert_eq!(next, Some("start".to_string()));
        assert_eq!(next_state.count, 10);
    }
}
