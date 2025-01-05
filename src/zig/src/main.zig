const std = @import("std");
const config = @import("config");
const Cfg = @import("Cfg.zig");
const Launcher = @import("Launcher.zig");

pub fn main() !void {
    const cfg = try Cfg.read_config("../../misc/FF78Launcher.toml");
    std.debug.print("{}", .{cfg});

    try Launcher.writeSoundConfig(cfg);
}
