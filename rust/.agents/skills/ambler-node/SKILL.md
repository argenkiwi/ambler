---
name: ambler-node
description: Creates a new Ambler node in src/nodes/ — with inline tests, using test-driven development. Use this whenever the user wants to add a node, step, or state to an Ambler Rust project — even if they phrase it as "add a step", "create a handler", or describe the behaviour without using the word "node". Also use when creating a node WITH tests or following TDD for a new node. For adding tests to an existing node that already has an implementation, use `/ambler-test` instead.
metadata:
  author: leandro
  version: "2.0-rust"
---

# Ambler Node (Rust)

Follow these steps to create a new node using test-driven development. Tests are written before the implementation.

A node is a boxed closure, `Node<S, K> = Box<dyn Fn(S) -> BoxFuture<'static, Next<S, K>> + Send + Sync>`, where `Next<S, K> = (Option<K>, S)`. Every node module exports the same four items: `Edges<K>`, `Utils`, `DefaultUtils` and `factory`.

## 1. Gather requirements

Before writing any code, determine:

- **Research Existing Nodes:** Check `specs/*.md` and `src/nodes/` to see if a node with similar behaviour already exists. It is always better to reuse or adapt an existing node than to create a duplicate.
- **Node name**: The purpose of the node (e.g., `retry`, `prompt`, `validate`). The file will be named `src/nodes/<name>.rs` (`snake_case`, no `_node` suffix).
- **State shape**: What fields does this node read or mutate? State is expressed as traits with getters and setters (e.g., `HasCount`). Shared state traits live in `src/nodes/mod.rs` — reuse an existing one where possible, otherwise add a new one there. Require only the traits the node actually touches.
- **Edges**: What named transitions can this node take? Each becomes an `on_<x>: Option<K>` field on the `Edges<K>` struct.
- **Utils**: What side-effectful operations does the node perform? List them (e.g., `print`, `read_line`, `sleep`, `random`). Each becomes a method on the `Utils` trait with a production implementation on `DefaultUtils`.
- **Sync vs async**: Does the node need `.await`? Determines whether the node returns `Box::pin(std::future::ready(...))` or `Box::pin(async move { ... })`, and whether any util method returns `BoxFuture<'static, T>`.
- **Behaviour**: What does the node do, and how does it choose which edge to follow?
- **Branches**: List every edge return the node can take (e.g., `on_success`, `on_error`). Each becomes one test case. The types and branches decided here are the contract — if you find you need to change them during implementation, restart from this step.

---

## 2. Create a skeleton `src/nodes/<name>.rs`

Register the module in `src/nodes/mod.rs` (keep the list alphabetical), and add any new state trait there alongside the existing ones:

```rust
pub mod count;
pub mod <name>;
pub mod start;
pub mod stop;

/// A state that has a count field.
pub trait HasCount {
    fn count(&self) -> i32;
    fn set_count(&mut self, count: i32);
}
```

Then write only the type exports and a stub factory. The node body must panic with `todo!()` so that tests fail in step 4.

**Synchronous skeleton:**

```rust
use crate::nodes::HasCount;
use crate::Node;
use std::sync::Arc;

pub struct Edges<K> {
    pub on_success: Option<K>, // from requirements
    pub on_error: Option<K>,
}

pub trait Utils: Send + Sync {
    fn print(&self, message: &str);
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
    fn print(&self, message: &str) {
        println!("{message}");
    }
}

/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(_edges: Edges<K>, _utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    Box::new(|_state: S| todo!())
}
```

**Asynchronous skeleton:**

```rust
use crate::nodes::HasCount;
use crate::Node;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::time::Duration;

pub struct Edges<K> {
    pub on_success: Option<K>,
    pub on_error: Option<K>,
}

pub trait Utils: Send + Sync {
    fn sleep(&self, ms: u64) -> BoxFuture<'static, ()>;
}

pub struct DefaultUtils;

impl Utils for DefaultUtils {
    fn sleep(&self, ms: u64) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(Duration::from_millis(ms)))
    }
}

/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(_edges: Edges<K>, _utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    Box::new(|_state: S| todo!())
}
```

Verify the skeleton compiles before writing tests:

```bash
cargo check
```

---

## 3. Write the tests in `src/nodes/<name>.rs`

Append an inline `#[cfg(test)] mod tests` block to the bottom of the node file. Write one `#[tokio::test]` per branch identified in step 1. All utils must be mocked — no real I/O, network, or timing.

**Synchronous node:**

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

    struct MockUtils;

    impl Utils for MockUtils {
        fn print(&self, _message: &str) {}
    }

    fn node() -> Node<MockState, &'static str> {
        factory(
            Edges {
                on_success: Some("NEXT"),
                on_error: Some("ERROR"),
            },
            Some(Arc::new(MockUtils)),
        )
    }

    #[tokio::test]
    async fn increments_count_and_transitions_to_on_success() {
        let (next, state) = node()(MockState { count: 1 }).await;

        assert_eq!(next, Some("NEXT"));
        assert_eq!(state.count, 2);
    }
}
```

**Asynchronous node** — identical, except async utils are mocked with an immediately ready future:

```rust
    struct MockUtils;

    impl Utils for MockUtils {
        fn sleep(&self, _ms: u64) -> BoxFuture<'static, ()> {
            Box::pin(async {})
        }
    }
```

To make a mock configurable per test, give it fields (e.g., `struct MockUtils { input: &'static str }`) and pass them through the `node(...)` helper. To assert on what a util was called with, record it behind a `Mutex` and keep an `Arc` to the mock (from `src/nodes/stop.rs`):

```rust
    use std::sync::Mutex;

    #[derive(Default)]
    struct MockUtils {
        message: Mutex<Option<String>>,
    }

    impl Utils for MockUtils {
        fn print(&self, message: &str) {
            *self.message.lock().unwrap() = Some(message.to_string());
        }
    }

    // In the test:
    let utils = Arc::new(MockUtils::default());
    let node: Node<MockState, &'static str> =
        factory(Edges { on_done: None }, Some(utils.clone()));
```

### Test rules

- **One test per branch** — one for each edge the node can return.
- **Mock all `Utils`** — a `MockUtils` struct implementing `Utils` deterministically, with no side effects.
- **Mock the state** — a `MockState` struct implementing only the traits the node requires.
- **Use `&'static str` IDs** such as `"NEXT"` and `"ERROR"` for `K`, and `None` for terminal edges.
- **Destructure the result** as `let (next, state) = ... .await;` and assert `next` for the next node ID and `state` for the updated state.
- **Test names**: `snake_case`, `<expected_behaviour>_<condition>` (e.g., `transitions_to_on_error_if_input_is_invalid`).
- **Always `#[tokio::test]` and `.await`** — every node returns a future, even synchronous ones.

---

## 4. Run tests — expect failure

```bash
cargo test nodes::<name>::
```

Tests must fail with a `not yet implemented` panic (or assertion errors). This confirms the tests are actually exercising the code path.

- If tests fail with `not yet implemented` → correct, proceed to step 5.
- If tests fail due to a **compile error** → the skeleton types don't match the test — fix the skeleton types, not the tests.
- If tests **pass** → the skeleton was too complete; ensure the node body is `todo!()`.

---

## 5. Implement `src/nodes/<name>.rs`

Replace the stub body with the real logic and drop the leading underscores from `edges` and `utils`. The type exports (`Edges`, `Utils`, `DefaultUtils`) and the factory signature stay exactly as written in step 2.

**Synchronous node** — the closure owns `edges` and `utils` and returns a ready future:

```rust
/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(edges: Edges<K>, utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    let utils = utils.unwrap_or_else(|| Arc::new(DefaultUtils));

    Box::new(move |mut state: S| {
        utils.print(&format!("Current count: {}", state.count()));
        state.set_count(state.count() + 1);
        Box::pin(std::future::ready((edges.on_success.clone(), state)))
    })
}
```

**Asynchronous node** — wrap `edges` in an `Arc` too, and clone both `Arc`s into each `async move` block so the future is `'static`:

```rust
/// Creates the node. `None` utils uses [`DefaultUtils`].
pub fn factory<S, K>(edges: Edges<K>, utils: Option<Arc<dyn Utils>>) -> Node<S, K>
where
    S: HasCount + Send + 'static,
    K: Clone + Send + Sync + 'static,
{
    let utils = utils.unwrap_or_else(|| Arc::new(DefaultUtils));
    let edges = Arc::new(edges);

    Box::new(move |mut state: S| {
        let utils = utils.clone();
        let edges = edges.clone();
        Box::pin(async move {
            utils.sleep(1000).await;
            state.set_count(state.count() + 1);
            (edges.on_success.clone(), state)
        })
    })
}
```

### Key rules

- **State ownership**: The node takes `state` by value and returns it. Update it only through the trait setters (e.g., `state.set_count(...)`) before returning — never through shared or global mutable state.
- **Edges**: Return `edges.on_<x>.clone()` — never construct a node ID inside the node.
- **Utils**: All side effects go through `utils.*`. Complex or reusable logic (e.g., LLM calls, file I/O, extra crates) should be moved to `src/utils/` via `/ambler-util`.
- **Termination**: Terminal nodes are wired with `None` in the walk (e.g., `stop::factory(stop::Edges { on_done: None }, None)`).
- **Do not modify the tests** between steps 4 and 6 — the tests define the contract; the implementation must satisfy them.

---

## 5a. Adapting Existing Nodes (Optional)

Nodes depend on state traits, not concrete fields, so reusing an existing node with a walk whose state uses different field names needs no adapter node. Implement the node's trait on the walk's `State`, mapping it to whichever field holds the value:

```rust
use ambler::nodes::HasCount;

struct State {
    total: i32,
}

impl HasCount for State {
    fn count(&self) -> i32 {
        self.total
    }
    fn set_count(&mut self, count: i32) {
        self.total = count;
    }
}
```

Only write a wrapping node if the existing node's behaviour itself must change, and then treat it as a new node (start again from step 1).

---

## 6. Run tests — must pass

```bash
cargo test nodes::<name>::
```

All tests must be green, with no compiler warnings. Fix the implementation (never the tests) until they pass.

---

## 7. Checklist before finishing

- [ ] `src/nodes/<name>.rs` exists and `pub mod <name>;` is in `src/nodes/mod.rs`.
- [ ] Any new shared state trait is defined in `src/nodes/mod.rs`.
- [ ] `Edges<K>` uses `on_<x>: Option<K>` fields.
- [ ] `factory(edges: Edges<K>, utils: Option<Arc<dyn Utils>>) -> Node<S, K>` is exported, with `S: <Traits> + Send + 'static` and `K: Clone + Send + Sync + 'static`.
- [ ] The state trait bounds are minimal.
- [ ] `DefaultUtils` provides real implementations, and `None` utils falls back to it.
- [ ] State is only changed through trait setters on the owned `state`.
- [ ] Shared logic is in `src/utils/`.
- [ ] The `#[cfg(test)] mod tests` block has one `#[tokio::test]` per branch.
- [ ] All `Utils` in tests are mocked.
- [ ] Tests were written (step 3) before the full implementation (step 5).
- [ ] Tests failed on step 4 before passing on step 6.
- [ ] No tests were modified between steps 4 and 6.
