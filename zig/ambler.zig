
const std = @import("std");

pub const Next = struct {
    run: *const fn () anyerror!?*const Next,
};

pub fn amble(initial: ?*const Next) anyerror!void {
    var next = initial;
    while (next) |n| {
        next = try n.run();
    }
}

pub fn ambleFrom(initial_run: *const fn () anyerror!?*const Next) anyerror!void {
    const initial_next = Next{ .run = initial_run };
    try amble(&initial_next);
}
