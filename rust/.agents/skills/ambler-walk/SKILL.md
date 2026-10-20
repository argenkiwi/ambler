---
name: ambler-walk
description: Creates a complete Ambler walk — the Rust binary that wires the nodes (src/bin/<name>.rs) and the Markdown spec (specs/<name>.md) — and ensures all required nodes exist. Use this whenever a user wants to add a new program or flow to an Ambler Rust project, even if they say "new walk", "add a program", "wire up these nodes", or just describe what they want the app to do.
metadata:
  author: leandro
  version: "2.0-rust"
---

# Ambler Walk (Rust)

This skill guides you in creating a complete Ambler walk. A walk is a state-machine program consisting of two files:

1. `src/bin/<name>.rs` — Rust binary defining the shared `State`, the `NodeId` enum, the wired node graph and the run loop.
2. `specs/<name>.md` — Markdown specification describing the shared state and the logic/transitions of each step.

---

## Step 1 — Identify the Walk

- Determine the walk name (lowercase `snake_case`, e.g. `my_walk`). The file will be `src/bin/<name>.rs`, and Cargo picks it up as the binary `<name>` automatically.
- Clarify the walk's purpose: what program does it implement?
- Identify the nodes (steps) required and their transitions.

---

## Step 2 — Research and Reuse

Before creating new nodes, research existing ones to promote reuse and consistency:

- **Review Existing Specs:** Browse `specs/*.md` to identify nodes that already implement the required logic (e.g., "Ollama Check", "Model Select"). This is more efficient than reading source code and helps maintain architectural alignment.
- **Check Node Implementation:** For each node identified, verify its existence in `src/nodes/<node_name>.rs` and that it is registered in `src/nodes/mod.rs`. Note which state traits (e.g., `HasCount`) each node requires — the walk's `State` must implement all of them.

---

## Step 3 — Create the Specification File (`specs/<name>.md`)

Create the Markdown spec using the `/ambler-spec` skill. This acts as the **blueprint** for the entire walk, defining the shared state and the logic for every node.

---

## Step 4 — Ensure Nodes Exist

For each node defined in the specification:

- If it does **not** exist in `src/nodes/<node_name>.rs`, create it using the `/ambler-node` skill. This runs a full TDD pass — skeleton, tests (red), implementation, tests (green) — before completing. No separate test step is needed.
- If the node already exists but has no `#[cfg(test)] mod tests` block, use the `/ambler-test` skill to retrofit coverage.

---

## Step 5 — Create the Binary File (`src/bin/<name>.rs`)

Example: `src/bin/counter.rs`.

```rust
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
```

**Key rules:**
- Import `Ambler` and `NodeThunk` from `ambler`, and each node module (plus the state traits it needs) from `ambler::nodes`.
- Define the `State` struct at the top of the file and implement every state trait required by the walk's nodes.
- Define a `NodeId` enum with one variant per node, deriving `Clone, Copy, Debug, Hash, PartialEq, Eq`.
- Build a `HashMap<NodeId, NodeThunk<State, NodeId>>` with `HashMap::from([...])`, mapping each ID to a **thunk that returns a node**: `Box::new(|| <node>::factory(<node>::Edges { ... }, None))`.
- Cast the first entry with `as NodeThunk<_, _>` so the array's element type becomes the boxed trait object; the remaining entries then coerce to it.
- Wire terminal edges with `None` (e.g., `stop::Edges { on_done: None }`). Pass `None` as utils to use each node's `DefaultUtils`.
- `Ambler::new(nodes)` builds each node lazily on its first `step` and caches it; an ID missing from the map panics with `Node not found: ...`, so every `NodeId` used in an edge must have an entry.
- Run the walk with `#[tokio::main]` and the loop `while let Some(id) = node_id { (node_id, state) = amble.step(id, state).await; }`, with `amble` declared `mut`.

---

## Step 6 — Check Dependencies and Arguments

Cargo discovers `src/bin/<name>.rs` automatically, so no `[[bin]]` entry is needed in `Cargo.toml`. Only touch `Cargo.toml` if the walk itself (not its nodes or utils) needs a new crate — add it with `cargo add <crate>`. Node and util dependencies are handled by `/ambler-node` and `/ambler-util`.

If the walk reads command-line arguments (e.g., via `std::env::args()` when building the initial `State`), users pass them after `--`:

```bash
cargo run --bin my_walk -- <arg1> <arg2>
```

---

## Step 7 — Verify

Run the walk to confirm it behaves as specified:

```bash
cargo run --bin <name>
```

If the walk has new nodes, also run:

```bash
cargo test
```

---

## Checklist

Before finishing, confirm:

- [ ] `specs/<name>.md` exists and matches the `NodeId` variants in the `.rs` file.
- [ ] `src/bin/<name>.rs` exists with the correct `State`, `NodeId`, wired node map and initial state.
- [ ] `State` implements every state trait the walk's nodes require.
- [ ] Every node used in the walk has a corresponding `src/nodes/<node_name>.rs`, registered in `src/nodes/mod.rs`, with passing tests (enforced by `/ambler-node`).
- [ ] Every `NodeId` referenced in an edge has an entry in the node map.
- [ ] All tests pass (`cargo test`) with no compiler warnings.
- [ ] The walk runs end-to-end without errors (`cargo run --bin <name>`).
