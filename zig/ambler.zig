
const std = @import("std");

pub fn resolve(comptime S: type, comptime A: type, comptime D: type, result: struct { S, A }, direct: *const fn (A) ?D) struct { S, ?D } {
    return .{ result.S, direct(result.A) };
}

pub fn amble(state: anytype, edge: anytype, follow: anytype) !void {
    var current_edge = edge;
    var current_state = state;
    while (current_edge) |e| {
        const result = try follow(current_state, e);
        current_state = result.state;
        current_edge = result.edge;
    }
}
