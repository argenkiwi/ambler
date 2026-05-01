use ambler::nodes::count_node;
use ambler::nodes::start_node::{self, HasCount};
use ambler::nodes::stop_node;
use ambler::{Ambler, Node};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct State {
    count: i32,
}

impl HasCount for State {
    fn count(&self) -> i32 {
        self.count
    }
    fn set_count(&mut self, count: i32) {
        self.count = count;
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum NodeId {
    Start,
    Count,
    Stop,
}

#[tokio::main]
async fn main() {
    let mut nodes: HashMap<NodeId, Box<dyn Node<State, NodeId>>> = HashMap::new();

    // Start Node
    let mut start_edges = HashMap::new();
    start_edges.insert(start_node::Edge::OnSuccess, Some(NodeId::Count));
    start_edges.insert(start_node::Edge::OnError, Some(NodeId::Start));
    nodes.insert(
        NodeId::Start,
        Box::new(start_node::StartNode::new(
            start_edges,
            Box::new(start_node::DefaultUtils),
        )),
    );

    // Count Node
    let mut count_edges = HashMap::new();
    count_edges.insert(count_node::Edge::OnCount, Some(NodeId::Count));
    count_edges.insert(count_node::Edge::OnStop, Some(NodeId::Stop));
    nodes.insert(
        NodeId::Count,
        Box::new(count_node::CountNode::new(
            count_edges,
            Box::new(count_node::DefaultUtils),
        )),
    );

    // Stop Node
    let mut stop_edges = HashMap::new();
    stop_edges.insert(stop_node::Edge::OnDone, None);
    nodes.insert(
        NodeId::Stop,
        Box::new(stop_node::StopNode::new(
            stop_edges,
            Box::new(stop_node::DefaultUtils),
        )),
    );

    let ambler = Ambler::new(nodes);

    let mut node_id = Some(NodeId::Start);
    let mut state = State { count: 0 };

    while let Some(id) = node_id {
        let (next_id, next_state) = ambler.step(id, state).await;
        node_id = next_id;
        state = next_state;
    }
}
