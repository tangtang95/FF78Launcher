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

    // Need to manually include C include path and library path due to zig unable to auto detect x86-windows-msvc paths
    lib.addIncludePath(.{ .cwd_relative = "C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.22621.0\\ucrt" });
    lib.addIncludePath(.{ .cwd_relative = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.38.33130/include" });

    lib.installHeader(upstream.path("Main/StackWalker/StackWalker.h"), "StackWalker.h");
    b.installArtifact(lib);
}
