# Ambler Application Guidelines

When building applications using `ambler.rs`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.rs`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `Start`: Initializes the process.
- `Step`: Performs the counting action.
- `Stop`: Terminates the process.

## 2. Define Nodes

Create an `enum`, typically named `Node`, to define these stages.

```rust
enum Node {
    Start,
    Step,
    Stop,
}
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an integer for a counter) or a more complex data structure (like a `struct` or a custom type).

In the counter example, the state is an integer representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return a tuple containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be `()` (unit type) if there's only one possible next step.

```rust
fn start(state: i32) -> (i32, ()) {
    println!("Let's count...");
    (state, ())
}

fn step(state: i32) -> (i32, bool) {
    let count = state + 1;
    println!("...{count}...");
    // Return a boolean to decide whether to continue
    (count, rand::random())
}

fn stop(state: i32) -> (i32, ()) {
    println!("...stop.");
    (state, ())
}
```

## 5. Create the `direct` Function

This function acts as the central router for your application. It takes the current `state` and `node` as input and uses the `ambler::resolve` function to determine the next step.

The `ambler::resolve` function takes two arguments:
1. The result of calling the appropriate node function (the `(state, value)` tuple).
2. A closure that takes the `value` from the node function's result and returns the next `Node` to execute.

```rust
use ambler::resolve;

async fn direct(state: i32, node: Node) -> (i32, Option<Node>) {
    match node {
        Node::Start => {
            // After 'start', always go to 'Step'
            resolve(start(state), |_| Some(Node::Step))
        }
        Node::Step => {
            // After 'step', check the boolean to either loop or stop
            resolve(step(state), |should_continue| {
                if should_continue {
                    Some(Node::Step)
                } else {
                    Some(Node::Stop)
                }
            })
        }
        Node::Stop => {
            // 'Stop' is the final node, so we return None
            resolve(stop(state), |_| None)
        }
    }
}
```

## 6. Start the Application

Finally, in your `main` function, call the `ambler::amble` function, passing the initial state, the starting node, and a reference to your `direct` function.

```rust
use ambler::amble;

#[tokio::main]
async fn main() {
    // Start with an initial state of 0 at the Start node
    amble(0, Node::Start, direct).await;
}
```
