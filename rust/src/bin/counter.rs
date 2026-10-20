use ambler::nodes::{count, start, stop, HasCount};
use ambler::{Ambler, NodeThunk};
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
    let nodes: HashMap<NodeId, NodeThunk<State, NodeId>> = HashMap::from([
        (
            NodeId::Start,
            Box::new(|| {
                start::factory(
                    start::Edges {
                        on_success: Some(NodeId::Count),
                        on_error: Some(NodeId::Start),
                    },
                    None,
                )
            }) as NodeThunk<_, _>,
        ),
        (
            NodeId::Count,
            Box::new(|| {
                count::factory(
                    count::Edges {
                        on_count: Some(NodeId::Count),
                        on_stop: Some(NodeId::Stop),
                    },
                    None,
                )
            }),
        ),
        (
            NodeId::Stop,
            Box::new(|| stop::factory(stop::Edges { on_done: None }, None)),
        ),
    ]);

    let mut amble = Ambler::new(nodes);

    let mut node_id = Some(NodeId::Start);
    let mut state = State { count: 0 };

    while let Some(id) = node_id {
        (node_id, state) = amble.step(id, state).await;
    }
}
