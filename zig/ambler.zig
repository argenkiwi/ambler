
const std = @import("std");

pub const Next = struct {
    run: *const fn () anyerror!?*const Next,
};

pub fn amble(initial: *const fn (i32) anyerror!?*const Next, state: i32) anyerror!void {
    var next = try initial(state);
    while (next) |n| {
        next = try n.run();
    }
}
