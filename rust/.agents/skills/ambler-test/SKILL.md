---
name: ambler-test
description: Adds or updates tests for an existing Ambler node that already has an implementation. Use this when a node exists in src/nodes/<name>.rs but has no #[cfg(test)] module, when tests need extending after new branches were added, or when retrofitting TDD onto existing code. For creating a brand-new node with tests, use `/ambler-node` instead.
metadata:
  author: leandro
  version: "2.0-rust"
---

# Ambler Test (Rust)

Follow these steps to add or update tests for an existing node in `src/nodes/`.

> For brand-new nodes, use `/ambler-node` — it creates both the implementation and tests via a TDD cycle. This skill is for retrofitting tests onto nodes that already exist.

---

## 1. Read the existing node

Read `src/nodes/<name>.rs` in full. Identify:

- **State traits** (the bounds on `S`, e.g. `S: HasCount + Send + 'static` — defined in `src/nodes/mod.rs`), the **`Edges<K>`** fields, and the **`Utils`** trait methods.
- **Sync vs async**: Does the node return `Box::pin(std::future::ready(...))` or `Box::pin(async move { ... })`? Do any util methods return `BoxFuture<'static, T>`?
- **All branches**: Every place the node returns `edges.on_<x>.clone()` is one branch. List them all — each becomes a test.

---

## 2. Check for existing tests

Check whether the node file already has a `#[cfg(test)] mod tests` block at the bottom.

- If it **does not exist**: write a complete new test module covering every branch.
- If it **does exist**: read it, identify which branches already have coverage, and list the gaps. Only add tests for uncovered branches — do not overwrite passing tests unless the node's interface changed.

---

## 3. Write or update the `#[cfg(test)] mod tests` block

Append the module to the bottom of `src/nodes/<name>.rs`, with one `#[tokio::test]` per uncovered branch. All utils must be mocked.

```rust
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
        input: &'static str,
    }

    impl Utils for MockUtils {
        // Override each util to be deterministic and side-effect-free.
        fn read_line(&self, _message: &str) -> Option<String> {
            Some(self.input.to_string())
        }
        fn print(&self, _message: &str) {}
    }

    fn node(input: &'static str) -> Node<MockState, &'static str> {
        factory(
            Edges {
                on_success: Some("NEXT"),
                on_error: Some("ERROR"),
            },
            Some(Arc::new(MockUtils { input })),
        )
    }

    #[tokio::test]
    async fn transitions_to_on_success_with_input_number() {
        let (next, state) = node("42")(MockState { count: 0 }).await;

        assert_eq!(next, Some("NEXT"));
        assert_eq!(state.count, 42);
    }

    #[tokio::test]
    async fn transitions_to_on_error_if_input_is_invalid() {
        let (next, state) = node("abc")(MockState { count: 123 }).await;

        assert_eq!(next, Some("ERROR"));
        assert_eq!(state.count, 123);
    }
}
```

For **async utils**, mock them with an immediately ready future (the node module already imports `BoxFuture`, so `use super::*;` brings it in):

```rust
    impl Utils for MockUtils {
        fn print(&self, _message: &str) {}
        fn sleep(&self, _ms: u64) -> BoxFuture<'static, ()> {
            Box::pin(async {})
        }
        fn random(&self) -> f64 {
            self.random
        }
    }
```

To assert on what a util received, record it behind a `std::sync::Mutex` in a `#[derive(Default)]` mock, keep an `Arc` to it, and pass `Some(utils.clone())` to `factory` (see `src/nodes/stop.rs`).

### Test rules

- **One test per branch** — cover every `edges.on_<x>.clone()` return.
- **Mock all `Utils`** — a `MockUtils` struct with no real I/O, network, or timing.
- **Mock the state** — a `MockState` struct implementing only the traits the node requires.
- **Use `&'static str` IDs** such as `"NEXT"` and `"ERROR"` for `K`, and `None` for terminal edges.
- **Destructure the result** as `let (next, state) = ... .await;` and assert `next` for the next node ID and `state` for the updated state.
- **Test names**: `snake_case`, `<expected_behaviour>_<condition>` (e.g., `transitions_to_on_error_if_input_is_invalid`).
- **Always `#[tokio::test]` and `.await`** — every node returns a future, even synchronous ones.

---

## 4. Run tests — must pass

```bash
cargo test nodes::<name>::
```

All tests must pass. If any fail, fix the implementation (not the tests).

---

## 5. Checklist before finishing

- [ ] `src/nodes/<name>.rs` ends with a `#[cfg(test)] mod tests` block.
- [ ] It has one `#[tokio::test]` per branch.
- [ ] `MockState` and `MockUtils` are defined — no real I/O, network, or timing.
- [ ] Every edge path has a dedicated test.
- [ ] Test names follow `<expected_behaviour>_<condition>` in `snake_case`.
- [ ] All tests pass (`cargo test`).
