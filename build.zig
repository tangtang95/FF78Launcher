const std = @import("std");

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{ .default_target = .{ .cpu_arch = .x86, .abi = .msvc, .os_tag = .windows } });
    const optimize = b.standardOptimizeOption(.{});

    const exe = b.addExecutable(.{
        .name = "FF78Launcher",
        .target = target,
        .optimize = optimize,
    });
    exe.linkLibC();

    const plog = b.dependency("plog", .{});
    const tomlplusplus = b.dependency("tomlplusplus", .{});
    const stackwalker = b.dependency("StackWalker", .{ .target = target, .optimize = optimize });

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

    // Need to manually include C include path and library path due to zig unable to auto detect x86-windows-msvc paths
    exe.addIncludePath(.{ .cwd_relative = "C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.22621.0\\ucrt" });
    exe.addIncludePath(.{ .cwd_relative = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.38.33130/include" });
    exe.addLibraryPath(.{ .cwd_relative = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.38.33130/Lib/x86" });
    exe.addLibraryPath(.{ .cwd_relative = "C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.22621.0\\um\\x86" });
    exe.addLibraryPath(.{ .cwd_relative = "C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.22621.0\\ucrt\\x86" });

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
            "-DAPP_RELEASE_NAME=\"FF78Launcher\"",
            "-DAPP_RELEASE_VERSION=\"0.0.0\"",
            "-DAPP_CONFIG_FILE=\"FF78Launcher.toml\"",
            "-DTOML_ENABLE_SIMD=0", // TODO: temporary disabling SIMD to build
            "-D_CRT_SECURE_NO_WARNINGS", // TODO: remove
            "-DNOMINMAX", // NOTE: is it really needed?
        },
    });
    // exe.addWin32ResourceFile(.{ .file = b.path("src/version.rc") });
    b.installArtifact(exe);
}
