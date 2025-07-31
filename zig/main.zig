const std = @import("std");
const ambler = @import("ambler.zig");

fn start(state: i32) !?*const ambler.Next {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Enter a starting number (or press Enter for default): ", .{});
    var buffer: [100]u8 = undefined;
    const input = try std.io.getStdIn().reader().readUntilDelimiterOrEof(&buffer, '\n');

    if (input) |i| {
        const trimmed = std.mem.trim(u8, i, " \t\r\n");
        if (trimmed.len == 0) {
            try stdout.print("Using default starting number.\n", .{});
            const next = ambler.Next{ .run = struct {
                pub fn run() !?*const ambler.Next {
                    return count(state);
                }
            }.run };
            return &next;
        } else {
            const number = std.fmt.parseInt(i32, trimmed, 10);
            if (number) |n| {
                const next = ambler.Next{ .run = struct {
                    pub fn run() !?*const ambler.Next {
                        return count(n);
                    }
                }.run };
                return &next;
            } else |_| {
                try stdout.print("Invalid number, please try again.\n", .{});
                const next = ambler.Next{ .run = struct {
                    pub fn run() !?*const ambler.Next {
                        return start(state);
                    }
                }.run };
                return &next;
            }
        }
    } else {
        return error.EndOfStream;
    }
}

fn count(state: i32) !?*const ambler.Next {
    std.debug.print("Count: {d}\n", .{state});
    std.time.sleep(1 * std.time.ns_per_s);
    const new_state = state + 1;
    var rand = std.rand.DefaultPrng.init(0);
    if (rand.random().float(f64) > 0.5) {
        const next = ambler.Next{ .run = struct {
            pub fn run() !?*const ambler.Next {
                return count(new_state);
            }
        }.run };
        return &next;
    } else {
        const next = ambler.Next{ .run = struct {
            pub fn run() !?*const ambler.Next {
                return stop(new_state);
            }
        }.run };
        return &next;
    }
}

fn stop(state: i32) !?*const ambler.Next {
    std.debug.print("Stopping count at {d}.\n", .{state});
    return null;
}

pub fn main() !void {
    try ambler.amble(&start, 0);
}
