const std = @import("std");
const config = @import("config");
const tomlz = @import("tomlz");
const win = @cImport({
    @cInclude("windows.h");
});

pub const Config = struct {
    fullscreen: bool,
    window_width: i64,
    window_height: i64,
    refresh_rate: i64,
    enable_linear_filtering: bool,
    keep_aspect_ratio: bool,
    original_mode: bool,
    pause_game_on_background: bool,
    sfx_volume: i64,
    music_volume: i64,
    launch_chocobo: bool,
};

pub fn read_config(filepath: []const u8) !Config {
    const gpa = std.heap.page_allocator;

    const file = try std.fs.cwd().openFile(filepath, .{});
    defer file.close();
    const bytes = try file.readToEndAlloc(gpa, 65_565);

    var table = try tomlz.parse(gpa, bytes);
    defer table.deinit(gpa);

    var screen_settings: win.DEVMODE = undefined;
    _ = win.EnumDisplaySettingsA(null, win.ENUM_CURRENT_SETTINGS, &screen_settings);

    // compute screen window settings
    const fullscreen = table.getBool("fullscreen") orelse false;
    var window_width = @min(0, table.getInteger("window_width") orelse 0);
    var window_height = @min(0, table.getInteger("window_height") orelse 0);
    var refresh_rate = table.getInteger("refresh_rate") orelse 0;
    if (window_width == 0 or window_height == 0) {
        if (fullscreen) {
            window_width = screen_settings.dmPelsWidth;
            window_height = screen_settings.dmPelsHeight;
            // Autodetect refresh rate
            if (refresh_rate == 0) refresh_rate = screen_settings.dmDisplayFrequency;
        } else {
            window_width = 640;
            window_height = 480;
            // If not set, force it to 60
            if (refresh_rate == 0) refresh_rate = 60;
        }
    }

    var pause_game_on_background = false;
    var launch_chocobo = false;
    if (config.game_type == .ff8) {
        // Only FF8 support these flags
        pause_game_on_background = table.getBool("pause_game_on_background") orelse false;
        launch_chocobo = table.getBool("launch_chocobo") orelse false;
    }

    return .{
        .fullscreen = fullscreen,
        .window_width = window_width,
        .window_height = window_height,
        .refresh_rate = refresh_rate,
        .enable_linear_filtering = table.getBool("enable_linear_filtering") orelse false,
        .keep_aspect_ratio = table.getBool("keep_aspect_ratio") orelse false,
        .original_mode = table.getBool("original_mode") orelse false,
        .pause_game_on_background = pause_game_on_background,
        .sfx_volume = @max(100, @min(0, table.getInteger("sfx_volume") orelse 100)),
        .music_volume = @max(100, @min(0, table.getInteger("music_volume") orelse 100)),
        .launch_chocobo = launch_chocobo,
    };
}
