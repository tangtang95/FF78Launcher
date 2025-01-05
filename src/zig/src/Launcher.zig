const std = @import("std");
const config = @import("config");
const Config = @import("Cfg.zig").Config;
const c = @cImport({
    @cInclude("shlwapi.h");
    @cInclude("shlobj.h");
});

const sound_config_file = switch (config.game_type) {
    .ff7 => "ff7sound.cfg",
    .ff8 => "ff8sound.cfg",
};
const video_config_file = switch (config.game_type) {
    .ff7 => "ff7video.cfg",
    .ff8 => "ff8video.cfg",
};

fn processGameMessages() u32 {
    std.debug.print("Starting game message queue thread...", .{});
}

fn getDocumentPath() !std.ArrayList(u8) {
    const gpa = std.heap.page_allocator;
    var doc_path = std.ArrayList(u8).init(gpa);

    const data_music_2 = std.fs.cwd().statFile("data/music_2") catch null;
    if (data_music_2 == null) { //TODO add ff7_estore_edition check
        var out_path: [256]u16 = undefined;
        const hr = c.SHGetKnownFolderPath(&c.FOLDERID_Documents, c.KF_FLAG_DEFAULT, null, @ptrCast(@alignCast(&out_path)));
        const path = try std.unicode.utf16LeToUtf8Alloc(gpa, &out_path);
        defer gpa.free(path);

        if (c.SUCCEEDED(hr)) {
            try doc_path.appendSlice(path);
            try doc_path.appendSlice("\\Square Enix\\FINAL FANTASY");
            switch (config.game_type) {
                .ff7 => try doc_path.appendSlice("VII Steam"),
                .ff8 => try doc_path.appendSlice("VIII Steam"),
            }
        }
    } else {
        _ = try std.fs.cwd().realpath("", doc_path.items);
    }

    return doc_path;
}

pub fn writeVideoConfig(cfg: Config) !void {
    var filepath = try getDocumentPath();
    defer filepath.deinit();

    try filepath.appendSlice(video_config_file);
    const file = try std.fs.cwd().openFile(filepath.items, .{ .mode = .read_write });
    defer file.close();
    try file.writer().writeInt(i32, @as(i32, cfg.window_width), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.window_height), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.refresh_rate), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.fullscreen), .little);
    try file.writer().writeInt(i32, 0, .little);
    try file.writer().writeInt(i32, @as(i32, cfg.keep_aspect_ratio), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.enable_linear_filtering), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.original_mode), .little);
    if (config.game_type == .ff8) {
        try file.writer().writeInt(i32, @as(i32, cfg.pause_game_on_background), .little);
    }
}

pub fn writeSoundConfig(cfg: Config) !void {
    var filepath = try getDocumentPath();
    defer filepath.deinit();

    try filepath.appendSlice(sound_config_file);
    const file = try std.fs.cwd().openFile(filepath.items, .{ .mode = .read_write });
    defer file.close();
    try file.writer().writeInt(i32, @as(i32, cfg.sfx_volume), .little);
    try file.writer().writeInt(i32, @as(i32, cfg.music_volume), .little);
}
