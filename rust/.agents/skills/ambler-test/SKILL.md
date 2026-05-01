name: ambler-test
description: Creates tests for an Ambler node. Use this whenever the user wants tests for a node — including "test this node", "add tests", "write tests for X", or any time a node is created without a corresponding test module.
metadata:
  author: leandro
  version: "1.0-rust"
---

# Ambler Test (Rust)

Follow these steps to create a test module for a node in its own file.

## 1. Gather requirements

Before writing any code, determine:

- **Node file**: The `src/nodes/<name>_node.rs` file.
- **Node's State, Edges, and Utils**: Read the node file to understand what the node does, which edges it has, and what utils it uses.
- **Branches to cover**: Every branch where `next_node` is determined is a test case.

---

## 2. Create the test module

Append a `#[cfg(test)]` module to the bottom of the node file.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockState {
        field: i32,
    }
    impl HasField for MockState {
        fn field(&self) -> i32 { self.field }
        fn set_field(&mut self, value: i32) { self.field = value; }
    }

    struct MockUtils;
    #[async_trait]
    impl Utils for MockUtils {
        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_my_node_success() {
        let mut edges = HashMap::new();
        edges.insert(Edge::OnSuccess, Some("next".to_string()));
        
        let node = MyNode::new(edges, Box::new(MockUtils));
        let state = MockState { field: 0 };
        
        let (next, next_state) = node.run(state, "start".to_string()).await;
        
        assert_eq!(next, Some("next".to_string()));
        assert_eq!(next_state.field, 1);
    }
}
```

### Test rules

- **Mock Utils**: Create a `MockUtils` struct that implements the `Utils` trait deterministically.
- **Mock State**: Create a `MockState` struct that implements the required state traits.
- **Async Tests**: Use `#[tokio::test]` and `async fn`.
- **Assertions**: Use `assert_eq!` to verify the next node ID and the updated state.
- **Coverage**: Ensure every edge/branch is tested.

---

## 3. Checklist before finishing

- [ ] `#[cfg(test)]` module exists at the bottom of the node file.
- [ ] `MockUtils` and `MockState` are implemented.
- [ ] All logic branches are covered by tests.
- [ ] Run `cargo test` to verify.
