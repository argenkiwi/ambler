name: ambler-node
description: Creates a new Ambler node in the nodes/ directory. Use this whenever the user wants to add a node, step, or state to an Ambler project — even if they phrase it as "add a step", "create a handler", or describe the behavior without using the word "node".
metadata:
  author: leandro
  version: "1.0-rust"
---

# Ambler Node (Rust)

Follow these steps to create a new node in the `src/nodes/` directory.

## 1. Gather requirements

Before writing any code, determine:

- **Node name**: The purpose of the node (e.g., `retry`, `prompt`, `validate`). The file will be named `<name>_node.rs`.
- **State shape**: What fields does this node read or mutate? Define a trait (or use an existing one) that the state must implement.
- **Edges**: What named transitions can this node take? Define an `Edge` enum.
- **Utils**: What side-effectful operations does the node perform? Define a `Utils` trait.
- **Behavior**: What does the node do, and how does it choose which Edge to follow?

---

## 2. Create `src/nodes/<name>_node.rs`

Use the following structure.

```rust
use crate::{Next, Node};
use async_trait::async_trait;
use std::collections::HashMap;

// 1. Define the State requirements as a trait
pub trait HasField {
    fn field(&self) -> i32;
    fn set_field(&mut self, value: i32);
}

// 2. Define the Utils trait
#[async_trait]
pub trait Utils: Send + Sync {
    fn log(&self, message: &str);
}

pub struct DefaultUtils;
impl Utils for DefaultUtils {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

// 3. Define the Edge enum
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Edge {
    OnSuccess,
    OnError,
}

// 4. Define the Node struct
pub struct MyNode<K> {
    pub edges: HashMap<Edge, Option<K>>,
    pub utils: Box<dyn Utils>,
}

impl<K> MyNode<K> {
    pub fn new(edges: HashMap<Edge, Option<K>>, utils: Box<dyn Utils>) -> Self {
        Self { edges, utils }
    }
}

// 5. Implement the Node trait
#[async_trait]
impl<S, K> Node<S, K> for MyNode<K>
where
    S: HasField + Send + Sync + 'static,
    K: Clone + Send + Sync + std::hash::Hash + std::cmp::Eq + 'static,
{
    async fn run(&self, mut state: S, _key: K) -> Next<S, K> {
        self.utils.log("Executing MyNode");
        
        // Logic here...
        state.set_field(state.field() + 1);
        
        let next_node = self.edges.get(&Edge::OnSuccess).cloned().flatten();
        (next_node, state)
    }
}
```

### Key rules

- **Naming**: Use `snake_case` for filenames (`<name>_node.rs`).
- **Traits**: Use traits for `Utils` and `State` requirements to maintain flexibility.
- **Async**: Use `#[async_trait]` for the `Node` and `Utils` traits.
- **Immutability/Mutation**: The `run` method takes `mut state: S`, allowing you to update it before returning.
- **Registration**: Remember to add `pub mod <name>_node;` to `src/nodes/mod.rs`.

---

## 3. Create Tests

Use the `/ambler-test` skill to generate the test file. In Rust, tests are usually in a `tests` module within the same file or in a separate `tests/` directory. For Ambler, we'll put them in a `tests` module at the bottom of the node file.

---

## 4. Checklist before finishing

- [ ] `src/nodes/<name>_node.rs` exists.
- [ ] `Edge` enum is defined.
- [ ] `Utils` trait and `DefaultUtils` exist.
- [ ] `Node` trait is implemented for the node struct.
- [ ] `src/nodes/mod.rs` is updated.
