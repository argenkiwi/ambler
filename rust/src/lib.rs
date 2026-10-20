use futures::future::BoxFuture;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod nodes;

/// The result of a node's execution: the next node ID (or `None` to stop) and the updated state.
pub type Next<S, K> = (Option<K>, S);

/// A processing step in the state machine.
///
/// Receives the current state and returns a future resolving to a [`Next`] tuple.
pub type Node<S, K> = Box<dyn Fn(S) -> BoxFuture<'static, Next<S, K>> + Send + Sync>;

/// A deferred constructor for a [`Node`], called once on first use.
pub type NodeThunk<S, K> = Box<dyn Fn() -> Node<S, K> + Send + Sync>;

/// A state machine runner built from a map of node thunks.
///
/// Each thunk is called once on first use; the resulting node is cached for subsequent calls.
pub struct Ambler<S, K> {
    nodes: HashMap<K, NodeThunk<S, K>>,
    loaded: HashMap<K, Node<S, K>>,
}

impl<S, K> Ambler<S, K>
where
    K: Hash + Eq + Clone + Debug,
{
    pub fn new(nodes: HashMap<K, NodeThunk<S, K>>) -> Self {
        Self {
            nodes,
            loaded: HashMap::new(),
        }
    }

    /// Runs the node identified by `node_id` against `state`.
    ///
    /// # Panics
    ///
    /// If `node_id` is not present in the node map.
    pub async fn step(&mut self, node_id: K, state: S) -> Next<S, K> {
        let Some(thunk) = self.nodes.get(&node_id) else {
            panic!("Node not found: {node_id:?}");
        };
        let node = self.loaded.entry(node_id).or_insert_with(|| thunk());
        node(state).await
    }
}
