const std = @import("std");

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{ .default_target = .{ .os_tag = .windows, .abi = .msvc }});
    const optimize = b.standardOptimizeOption(.{});

    const upstream = b.dependency("StackWalker", .{});
    const lib = b.addStaticLibrary(.{
        .name = "StackWalker",
        .target = target,
        .optimize = optimize,
    });
    lib.linkLibC();

    lib.addIncludePath(upstream.path("Main/StackWalker/"));
    lib.addCSourceFile(.{ .file = upstream.path("Main/StackWalker/StackWalker.cpp") });

    lib.installHeader(upstream.path("Main/StackWalker/StackWalker.h"), "StackWalker.h");
    b.installArtifact(lib);
}
