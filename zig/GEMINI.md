# Ambler Application Guidelines

When building applications using `ambler.zig`, please adhere to the following guidelines to ensure consistency and maintainability.

> IMPORTANT: Never modify the code in `ambler.zig`!

## 1. Identify Program Stages (Nodes)

First, break down your program's logic into distinct steps or stages. These will become the "nodes" of your application's flow.

For example, a simple counter application might have the following stages:
- `Start`: Initializes the process.
- `Step`: Performs the counting action.
- `Stop`: Terminates the process.

## 2. Define Nodes

Create an `enum`, typically named `Node`, to define these stages.

```zig
const Node = enum {
    Start,
    Step,
    Stop,
};
```

## 3. Determine Shared State

Decide what data needs to be passed between the different nodes. This will be your "state" object. It can be a simple type (like an `i32` for a counter) or a more complex data structure (like a `struct` or a custom type).

In the counter example, the state is an integer representing the current count.

## 4. Create Node Functions

For each node, create a corresponding function that takes the current `state` as a parameter. Each function should return a `struct` containing:
1. The (potentially modified) `state`.
2. A value that will be used to decide which node to go to next. This can be an empty `struct {}` if there's only one possible next step.

```zig
const std = @import("std");

fn start(state: i32) struct { i32, void } {
    std.debug.print("Let's count...\n", .{});
    return .{ state, {} };
}

fn step(state: i32) struct { i32, bool } {
    const count = state + 1;
    std.debug.print("...{}\n", .{count});
    // Return a boolean to decide whether to continue
    return .{ count, std.crypto.random.boolean() };
}

fn stop(state: i32) struct { i32, void } {
    std.debug.print("...stop.\n", .{});
    return .{ state, {} };
}
```

## 5. Create the `direct` Function

This function acts as the central router for your application. It takes the current `state` and `node` as input and uses the `ambler.resolve` function to determine the next step.

The `ambler.resolve` function takes three comptime type arguments (State, Action, Destination Node) and two runtime arguments:
1. The result of calling the appropriate node function (a `struct` containing `state` and `action`).
2. A function pointer that takes the `action` from the node function's result and returns the next `Node` to execute (or `null` if it's the final node).

```zig
const ambler = @import("ambler.zig");

fn direct(state: i32, node: Node) struct { i32, ?Node } {
    return switch (node) {
        .Start => ambler.resolve(i32, void, Node, start(state), (struct {
            fn(void) ?Node { return .Step; }
        }).comptime_field_0),
        .Step => ambler.resolve(i32, bool, Node, step(state), (struct {
            fn(bool) ?Node { |should_continue| return if (should_continue) .Step else .Stop; }
        }).comptime_field_0),
        .Stop => ambler.resolve(i32, void, Node, stop(state), (struct {
            fn(void) ?Node { return null; }
        }).comptime_field_0),
    };
}
```

## 6. Start the Application

Finally, in your `main` function, call the `ambler.amble` function, passing the initial state, the starting node, and a reference to your `direct` function.

```zig
const ambler = @import("ambler.zig");

pub fn main() !void {
    // Start with an initial state of 0 at the Start node
    try ambler.amble(0, Node.Start, direct);
}
```