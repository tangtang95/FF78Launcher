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
    setLibcWithXWin(b, lib);

    lib.addIncludePath(upstream.path("Main/StackWalker/"));
    lib.addCSourceFile(.{ .file = upstream.path("Main/StackWalker/StackWalker.cpp") });

    lib.installHeader(upstream.path("Main/StackWalker/StackWalker.h"), "StackWalker.h");
    b.installArtifact(lib);
}

fn setLibcWithXWin(b: *std.Build, exe: *std.Build.Step.Compile) void {
    const arch: []const u8 = switch (exe.rootModuleTarget().cpu.arch) {
        .x86 => "x86",
        else => @panic("Unsupported Architecture"),
    };

    exe.setLibCFile(b.path("libc.txt"));
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/crt/include") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/ucrt") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/um") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/10.0.26100/cppwinrt") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/10.0.26100/ucrt") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/10.0.26100/um") });
    exe.addSystemIncludePath(.{ .cwd_relative = sdkPath("/.xwin/sdk/include/10.0.26100/shared") });
    exe.addLibraryPath(.{ .cwd_relative = b.fmt(sdkPath("/.xwin/crt/lib/{s}"), .{arch}) });
    exe.addLibraryPath(.{ .cwd_relative = b.fmt(sdkPath("/.xwin/sdk/lib/ucrt/{s}"), .{arch}) });
    exe.addLibraryPath(.{ .cwd_relative = b.fmt(sdkPath("/.xwin/sdk/lib/um/{s}"), .{arch}) });
}

fn sdkPath(comptime suffix: []const u8) []const u8 {
    if (suffix[0] != '/') @compileError("relToPath requires an absolute path!");
    return comptime blk: {
        @setEvalBranchQuota(2000);
        const root_dir = std.fs.path.dirname(@src().file) orelse ".";
        break :blk root_dir ++ suffix;
    };
}
