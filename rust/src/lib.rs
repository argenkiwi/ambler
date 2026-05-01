use std::collections::HashMap;
use async_trait::async_trait;

pub mod nodes;

/// Represents the result of a node execution: (next_node_id, next_state)
pub type Next<S, K> = (Option<K>, S);

/// A trait representing a node in the state machine.
#[async_trait]
pub trait Node<S, K>: Send + Sync {
    async fn run(&self, state: S, key: K) -> Next<S, K>;
}

/// The single-step executor for the state machine.
pub struct Ambler<S, K>
where
    K: std::hash::Hash + std::cmp::Eq + Clone + Send + Sync,
    S: Send + Sync,
{
    nodes: HashMap<K, Box<dyn Node<S, K>>>,
}

impl<S, K> Ambler<S, K>
where
    K: std::hash::Hash + std::cmp::Eq + Clone + Send + Sync + 'static,
    S: Send + Sync + 'static,
{
    pub fn new(nodes: HashMap<K, Box<dyn Node<S, K>>>) -> Self {
        Self { nodes }
    }

    pub async fn step(&self, node_id: K, state: S) -> Next<S, K> {
        let node = self.nodes.get(&node_id).expect("Node not found");
        node.run(state, node_id).await
    }
}
