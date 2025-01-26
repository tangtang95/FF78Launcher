const std = @import("std");
const GameType = @import("src/Types.zig").GameType;

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const build_for_ff8 = b.option(bool, "ff8", "Build for FF8, otherwise it will build for FF7") orelse false;

    const tomlz = b.dependency("tomlz", .{ .target = target, .optimize = optimize });

    const exe = b.addExecutable(.{ .name = "ff78-lib", .target = target, .optimize = optimize, .root_source_file = b.path("src/main.zig") });
    exe.linkLibC();

    exe.root_module.addImport("tomlz", tomlz.module("tomlz"));

    const options = b.addOptions();
    options.addOption(GameType, "game_type", if (build_for_ff8) GameType.ff8 else GameType.ff7);
    exe.root_module.addOptions("config", options);

    b.installArtifact(exe);

    const run_exe = b.addRunArtifact(exe);
    const run_step = b.step("run", "run executable");
    run_step.dependOn(&run_exe.step);

    const check_step = b.step("check", "check if compiles");
    check_step.dependOn(&exe.step);
}
