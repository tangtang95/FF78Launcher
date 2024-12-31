const std = @import("std");

pub fn build(b: *std.Build) !void {
    const app_name = "FF78Launcher";
    const target = b.standardTargetOptions(.{ .default_target = .{ .cpu_arch = .x86, .abi = .msvc } });
    const optimize = b.standardOptimizeOption(.{});
    if (target.result.os.tag != .windows or target.result.abi != .msvc) {
        @panic("build supported only for windows msvc abi");
    }

    const plog = b.dependency("plog", .{});
    const tomlplusplus = b.dependency("tomlplusplus", .{});
    const stackwalker = b.dependency("StackWalker", .{ .target = target, .optimize = optimize });

    const exe = b.addExecutable(.{
        .name = app_name,
        .target = target,
        .optimize = optimize,
        .version = .{ .major = 0, .minor = 3, .patch = 0 },
    });
    exe.addWin32ResourceFile(.{ .file = b.path("src/version.rc"), .flags = &.{
        "/dVER_FILEVERSION=0.3.0",
        "/dVER_FILEVERSION_STR=\"0.3.0\\0\"",
        "/dVER_PRODUCTNAME=\"" ++ app_name ++ "\"",
        "/dVER_ORIGINALFILENAME=\"" ++ app_name ++ ".exe\"",
    } });
    exe.want_lto = false;

    // for compiling to a different arch than native (e.g. x86 on x86_64 machine), use xwin
    if(exe.rootModuleTarget().cpu.arch != b.graph.host.result.cpu.arch) {
        setLibcWithXWin(b, exe);
    }
    exe.linkLibC();

    exe.addIncludePath(plog.path("include"));
    exe.addIncludePath(tomlplusplus.path("include"));
    exe.addIncludePath(stackwalker.path(""));
    exe.addIncludePath(b.path("src"));
    exe.linkLibrary(stackwalker.artifact("StackWalker"));
    exe.linkSystemLibrary("shlwapi");
    exe.linkSystemLibrary("ole32");
    exe.linkSystemLibrary("advapi32");
    exe.linkSystemLibrary("user32");
    exe.linkSystemLibrary("shell32");

    exe.addCSourceFiles(.{
        .root = b.path("src"),
        .files = &.{
            "launcher.cpp",
            "winmain.cpp",
            "cfg.cpp",
        },
        .flags = &.{
            "-std=c++20",
            "-DWIN32",
            "-DWIN32_WINNT=0x0601",
            "-DAPP_RELEASE_NAME=\"" ++ app_name ++ "\"",
            "-DAPP_RELEASE_VERSION=\"0.3.0\"",
            "-DAPP_CONFIG_FILE=\"" ++ app_name ++ ".toml\"",
            "-DTOML_ENABLE_SIMD=0", // TODO: temporary disabling SIMD to build
            "-D_CRT_SECURE_NO_WARNINGS", // TODO: remove
            "-DNOMINMAX", // NOTE: is it really needed?
        },
    });

    b.installArtifact(exe);
    b.installFile("LICENSE", "bin/COPYING.txt");
    b.installFile("misc/FF78Launcher.toml", "bin/" ++ app_name ++ ".toml");
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
