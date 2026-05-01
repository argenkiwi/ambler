name: ambler-walk
description: Creates a complete Ambler walk — the Rust binary (src/bin/<name>.rs) and the Markdown spec (specs/<name>.md) — and ensures all required nodes exist.
metadata:
  author: leandro
  version: "1.0-rust"
---

# Ambler Walk (Rust)

This skill guides you in creating a complete Ambler walk in Rust.

---

## Step 1 — Identify the Walk

- Determine the walk name (snake_case, e.g. `my_walk`). The file will be `src/bin/<name>.rs`.
- Identify the nodes required and their transitions.

---

## Step 2 — Ensure Nodes Exist

For each node the walk requires:

- Check if `src/nodes/<node_name>_node.rs` exists.
- If not, create it using the `/ambler-node` skill.
- Ensure tests exist using `/ambler-test`.

---

## Step 3 — Create the Specification File (`specs/<name>.md`)

Create the Markdown spec using the `/ambler-spec` skill.

---

## Step 4 — Create the Binary File (`src/bin/<name>.rs`)

```rust
use ambler::nodes::{node_a, node_b};
use ambler::{Ambler, Node};
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct State {
    field: i32,
}
// Implement required traits for State...

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum NodeId {
    Start,
    Next,
}

#[tokio::main]
async fn main() {
    let mut nodes: HashMap<NodeId, Box<dyn Node<State, NodeId>>> = HashMap::new();

    // Wire nodes...
    
    let ambler = Ambler::new(nodes);
    let mut node_id = Some(NodeId::Start);
    let mut state = State { field: 0 };

    while let Some(id) = node_id {
        let (next_id, next_state) = ambler.step(id, state).await;
        node_id = next_id;
        state = next_state;
    }
}
```

---

## Step 5 — Verify

Run the walk:

```bash
cargo run --bin <name>
```

---

## Checklist

- [ ] `specs/<name>.md` exists.
- [ ] `src/bin/<name>.rs` exists and wires the nodes.
- [ ] All nodes and tests exist.
- [ ] All tests pass (`cargo test`).
- [ ] The walk runs end-to-end.
