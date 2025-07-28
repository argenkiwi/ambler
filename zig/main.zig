const std = @import("std");
const ambler = @import("ambler.zig");

// Nodes.
const Node = enum {
    Start,
    Step,
    Stop,
};

fn start(state: i32) struct { i32, void } {
    std.debug.print("Let's count...\n", .{});
    return .{ state, {} };
}

fn step(state: i32) struct { i32, bool } {
    const count = state + 1;
    std.debug.print("...{}...\n", .{count});
    return .{ count, std.crypto.random.boolean() };
}

fn stop(state: i32) struct { i32, void } {
    std.debug.print("...stop.\n", .{});
    return .{ state, {} };
}

// Flow.
fn direct(state: i32, node: Node) struct { i32, ?Node } {
    return switch (node) {
        .Start => ambler.resolve(i32, void, Node, start(state), (struct { 
            fn(void) ?Node { return .Step; }
        }). comptime_field_0),
        .Step => ambler.resolve(i32, bool, Node, step(state), (struct {
            fn(bool) ?Node { |should_continue| return if (should_continue) .Step else .Stop; }
        }). comptime_field_0),
        .Stop => ambler.resolve(i32, void, Node, stop(state), (struct {
            fn(void) ?Node { return null; }
        }). comptime_field_0),
    };
}

pub fn main() !void {
    try ambler.amble(0, Node.Start, direct);
}
