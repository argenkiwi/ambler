const std = @import("std");
const ambler = @import("ambler.zig");

fn promptForNumber() !i32 {
    const stdin = std.io.getStdIn().reader();
    const stdout = std.io.getStdOut().writer();
    var buffer: [100]u8 = undefined;

    while (true) {
        try stdout.print("Enter a starting number: ", .{});
        const input = try stdin.readUntilDelimiterOrEof(&buffer, '\n');
        if (input) |i| {
            const trimmed = std.mem.trim(u8, i, " \t\r\n");
            const number = std.fmt.parseInt(i32, trimmed, 10);
            if (number) |n| {
                return n;
            } else |_| {
                try stdout.print("Invalid number, please try again.\n", .{});
            }
        } else {
            return error.EndOfStream;
        }
    }
}

fn promptNumberNode(state: i32) !?*const ambler.Next {
    _ = state;
    const number = try promptForNumber();
    const next = ambler.Next{ .run = struct { 
        pub fn run() !?*const ambler.Next {
            return startNode(number);
        }
    }.run };
    return &next;
}

fn startNode(state: i32) !?*const ambler.Next {
    std.debug.print("Starting count from {d}\n", .{state});
    const next = ambler.Next{ .run = struct {
        pub fn run() !?*const ambler.Next {
            return stepNode(state);
        }
    }.run };
    return &next;
}

fn stepNode(state: i32) !?*const ambler.Next {
    const new_state = state + 1;
    std.debug.print("Count: {d}\n", .{new_state});
    var rand = std.rand.DefaultPrng.init(0);
    if (rand.random().float(f64) > 0.5) {
        const next = ambler.Next{ .run = struct {
            pub fn run() !?*const ambler.Next {
                return stepNode(new_state);
            }
        }.run };
        return &next;
    } else {
        const next = ambler.Next{ .run = struct {
            pub fn run() !?*const ambler.Next {
                return stopNode(new_state);
            }
        }.run };
        return &next;
    }
}

fn stopNode(state: i32) !?*const ambler.Next {
    _ = state;
    std.debug.print("Stopping count.\n", .{});
    return null;
}

pub fn main() !void {
    try ambler.ambleFrom(struct{
        pub fn run() !?*const ambler.Next {
            return promptNumberNode(0);
        }
    }.run);
}