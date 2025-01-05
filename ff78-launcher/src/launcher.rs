use std::io::Write;

use anyhow::Result;
use windows::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_Documents, SHGetKnownFolderPath, KF_FLAG_DEFAULT},
};

use crate::{Context, GameType, StoreType};

const FF7_USER_SAVE_DIR: u32 = 10;
const FF7_DOC_DIR: u32 = 11;
const FF7_INSTALL_DIR: u32 = 12;
const FF7_LOCALE_DATA_DIR: u32 = 13;
const FF7_GAME_VERSION: u32 = 18;
const FF7_DISABLE_CLOUD: u32 = 22;
const FF7_END_USER_INFO: u32 = 24;

const FF8_USER_SAVE_DIR: u32 = 9;
const FF8_DOC_DIR: u32 = 10;
const FF8_INSTALL_DIR: u32 = 11;
const FF8_LOCALE_DATA_DIR: u32 = 12;
const FF8_GAME_VERSION: u32 = 17;
const FF8_DISABLE_CLOUD: u32 = 21;
const FF8_BG_PAUSE_ENABLED: u32 = 23;
const FF8_END_USER_INFO: u32 = 24;

const ESTORE_USER_SAVE_DIR: u32 = 9;
const ESTORE_DOC_DIR: u32 = 10;
const ESTORE_INSTALL_DIR: u32 = 11;
const ESTORE_LOCALE_DATA_DIR: u32 = 12;
const ESTORE_GAME_VERSION: u32 = 17;
const ESTORE_END_USER_INFO: u32 = 20;

static mut LAUNCHER_MEMORY_PART: Vec<u8> = Vec::new();

pub fn get_game_install_path(ctx: &Context) -> Result<String> {
    let mut game_install_path = String::new();
    if !matches!(ctx.game_to_launch, GameType::FF7(StoreType::EStore))
        && std::fs::exists("data/music_2").is_err()
    {
        let doc_path = unsafe {
            let doc_path_pw = SHGetKnownFolderPath(&FOLDERID_Documents, KF_FLAG_DEFAULT, None)?;
            let doc_path = doc_path_pw.to_string()?;
            CoTaskMemFree(Some(doc_path_pw.as_ptr() as *const std::ffi::c_void));
            doc_path
        };
        game_install_path += &doc_path;
        game_install_path += "\\Square Enix\\FINAL FANTASY ";
        game_install_path += match ctx.game_to_launch {
            GameType::FF7(_) => "VII Steam",
            GameType::FF8 => "VIII Steam",
        }
    } else {
        let cwd = std::env::current_dir()?
            .to_str()
            .ok_or(anyhow::anyhow!("cwd cannot be converted to string"))?
            .to_string();
        game_install_path += &cwd;
    }
    Ok(game_install_path)
}

pub fn send_locale_data_dir(ctx: &Context) {
    let mem_payload = String::from("lang-") + &ctx.game_lang;
    let mut launcher_game_part = Vec::<u8>::new();
    launcher_game_part.push(match ctx.game_to_launch {
        GameType::FF7(StoreType::Standard) => FF7_LOCALE_DATA_DIR as u8,
        GameType::FF7(StoreType::EStore) => ESTORE_LOCALE_DATA_DIR as u8,
        GameType::FF8 => FF8_LOCALE_DATA_DIR as u8,
    });
    launcher_game_part.append(&mut (mem_payload.len() as u32).to_le_bytes().to_vec());
    launcher_game_part.append(
        &mut mem_payload
            .encode_utf16()
            .flat_map(|b| b.to_le_bytes())
            .collect(),
    );
    launcher_game_part.push(0);
    unsafe {
        // NOTE: Unsafe since single threaded
        LAUNCHER_MEMORY_PART = launcher_game_part;
    };
    log::info!("send_locale_data_dir -> {mem_payload}");

    // TODO:
}

pub fn write_ffvideo(ctx: &Context) -> Result<()> {
    let filename = match ctx.game_to_launch {
        GameType::FF7(_) => "ff7video.cfg",
        GameType::FF8 => "ff8video.cfg",
    };
    let filepath = get_game_install_path(ctx)? + "\\" + filename;
    let mut file = std::fs::File::create(filepath)?;
    file.write_all(&ctx.config.window_width.to_le_bytes())?;
    file.write_all(&ctx.config.window_height.to_le_bytes())?;
    file.write_all(&ctx.config.refresh_rate.to_le_bytes())?;
    file.write_all(&u32::from(ctx.config.fullscreen).to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&u32::from(ctx.config.keep_aspect_ratio).to_le_bytes())?;
    file.write_all(&u32::from(ctx.config.enable_linear_filtering).to_le_bytes())?;
    file.write_all(&u32::from(ctx.config.original_mode).to_le_bytes())?;
    if let GameType::FF8 = ctx.game_to_launch {
        file.write_all(&u32::from(ctx.config.pause_game_on_background).to_le_bytes())?;
    }
    Ok(())
}

pub fn write_ffsound(ctx: &Context) -> Result<()> {
    let filename = match ctx.game_to_launch {
        GameType::FF7(_) => "ff7sound.cfg",
        GameType::FF8 => "ff8sound.cfg",
    };
    let filepath = get_game_install_path(ctx)? + "\\" + filename;
    let mut file = std::fs::File::create(filepath)?;
    file.write_all(&ctx.config.sfx_volume.to_le_bytes())?;
    file.write_all(&ctx.config.music_volume.to_le_bytes())?;
    Ok(())
}
