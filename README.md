![License](https://img.shields.io/github/license/julianxhokaxhiu/FF78Launcher) ![Overall Downloads](https://img.shields.io/github/downloads/julianxhokaxhiu/FF78Launcher/total?label=Overall%20Downloads) ![Latest Stable Downloads](https://img.shields.io/github/downloads/julianxhokaxhiu/FF78Launcher/latest/total?label=Latest%20Stable%20Downloads&sort=semver) ![Latest Canary Downloads](https://img.shields.io/github/downloads/julianxhokaxhiu/FF78Launcher/canary/total?label=Latest%20Canary%20Downloads) ![GitHub Actions Workflow Status](https://github.com/julianxhokaxhiu/FF78Launcher/actions/workflows/main-0.3.0.yml/badge.svg?branch=master)

# FF78Launcher

An alternative launcher for FF7/FF8 Steam and eStore editions

## How to install

> **Please note:** Make a backup of your launcher or use the Steam Game files integrity check if something goes wrong and you're not able to launch the game.

1. Download the latest release from https://github.com/julianxhokaxhiu/FF78Launcher/releases
2. Extract it to your target FF7 or FF8 Steam folder.
3. Replace all files when asked.
4. You can now run the game via Steam as usual. Enjoy!

**REMEMBER:** Cloud sync is and always will be disabled. Please do not raise an issue for it, we are not going to implement it and we are not interested.

## How to change settings

After you install the launcher, you will find a file named [`FF78Launcher.toml`](misc/FF78Launcher.toml). Feel free to have a look at the file to know what can you change.

## How to build

### Requirements

- `zig` compiler (tested with 0.14.0-dev.2569+30169d1d2)
- `xwin` tool for local development on non-x86 machine

Download microsoft CRT headers and libraries, and Windows SDK headers and libraries:
```
xwin --arch=x86 --accept-license splat --output .xwin --include-debug-libs --include-debug-symbols --preserve-ms-arch-notation --disable-symlinks
```

### Build steps
```sh
# To build for debug mode
zig build
# To build for release fast mode
zig build --release=fast
```

## Supported languages

For a full list of supported game launchers, see [this list](https://github.com/julianxhokaxhiu/FF78Launcher/blob/master/src/winmain.cpp#L23-L37)

## How to launch the Chocobo exe

If you want to launch the chocobo exe while using this custom launcher, please set the `launch_chocobo` flag to `true` in the [`FF78Launcher.toml`](misc/FF78Launcher.toml) config file. When you want to launch back the game, set back the `launch_chocobo` flag to `false`.

## Known issues

- Controller settings are not inherited from the official launcher
