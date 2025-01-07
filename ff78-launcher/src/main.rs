#![windows_subsystem = "windows"]

use std::{os::windows::fs::MetadataExt, path::PathBuf};

use anyhow::Result;
use config::Config;
use launcher::{write_ffsound, write_ffvideo};
use log::LevelFilter;
use windows::{
    core::{s, PCSTR, PSTR},
    Win32::{
        Foundation::CloseHandle,
        System::{
            Diagnostics::Debug::{
                SetUnhandledExceptionFilter, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_POINTERS,
            },
            Threading::{
                CreateProcessA, CreateSemaphoreA, WaitForSingleObject, INFINITE,
                PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOA,
            },
        },
        UI::WindowsAndMessaging::{MessageBoxA, MB_ICONERROR, MB_OK},
    },
};

mod config;
mod launcher;

const APP_NAME: &str = "FF78Launcher";
const LOG_FILE: &str = "FF78Launcher.log";
const PROCESSES: [&str; 11] = [
    // FF7
    "ff7_de.exe",
    "ff7_en.exe",
    "ff7_es.exe",
    "ff7_fr.exe",
    "ff7_ja.exe",
    // FF8
    "ff8_de.exe",
    "ff8_en.exe",
    "ff8_es.exe",
    "ff8_fr.exe",
    "ff8_it.exe",
    "ff8_ja.exe",
];
const AF3DN_FILE: &str = "AF3DN.P";

static mut HAD_EXCEPTION: bool = false;

#[derive(Debug)]
enum StoreType {
    Standard,
    EStore,
}

#[derive(Debug)]
enum GameType {
    FF7(StoreType),
    FF8,
}

#[derive(Debug)]
pub struct Context {
    game_to_launch: GameType,
    game_lang: String,
    use_ffnx: bool,
    config: Config,
}

fn main() -> Result<()> {
    simple_logging::log_to_file(LOG_FILE, LevelFilter::Info)?;
    log::info!("{APP_NAME} launched!");

    unsafe {
        SetUnhandledExceptionFilter(Some(exception_handler));
    };

    let processes_available: Vec<&str> = PROCESSES
        .into_iter()
        .filter(|process| matches!(std::fs::exists(process), Ok(true)))
        .collect();
    if processes_available.len() > 1 {
        log::error!(
            "More than one process to start found: {:?}",
            processes_available
        );
        return Err(anyhow::anyhow!("No process to start found"));
    }
    let Some(mut process_to_start) = processes_available.first().map(|s| s.to_string()) else {
        log::error!("No process to start found!");
        return Err(anyhow::anyhow!("No process to start found!"));
    };

    let game_to_launch = match &process_to_start {
        name if name.starts_with("ff8") => GameType::FF8,
        name if name.starts_with("ff7_ja")
            && std::fs::metadata(AF3DN_FILE)
                .is_ok_and(|metadata| metadata.file_size() < 1024 * 1024) =>
        {
            GameType::FF7(StoreType::EStore)
        }
        _ => GameType::FF7(StoreType::Standard),
    };

    let use_ffnx =
        std::fs::metadata(AF3DN_FILE).is_ok_and(|metadata| metadata.file_size() > 1024 * 1024);
    let game_lang = process_to_start
        .split('_')
        .take(2)
        .last()
        .map(|end| end.trim_end_matches(".exe").to_string());
    let Some(game_lang) = game_lang else {
        log::error!(
            "No language found for process to start: {}",
            process_to_start
        );
        return Err(anyhow::anyhow!("No language found!"));
    };

    let config = Config::from_config_file(&(APP_NAME.to_string() + ".toml"), &game_to_launch)?;
    log::info!("config: {:?}", config);

    if config.launch_chocobo {
        process_to_start = format!("chocobo_{}.exe", &game_lang);
    }

    let ctx = Context {
        game_to_launch,
        game_lang: game_lang.to_string(),
        use_ffnx,
        config,
    };

    let process_to_start_path = std::fs::canonicalize(&process_to_start)?;
    if !use_ffnx || ctx.config.launch_chocobo {
        log::info!(
            "Launching process {:?} without FFNx context: {:?}",
            process_to_start_path,
            &ctx
        );
        if !use_ffnx {
            write_ffvideo(&ctx)?;
            write_ffsound(&ctx)?;
        }
        unsafe {
            let game_read_sem = CreateSemaphoreA(None, 0, 1, PCSTR("test".as_ptr()))?;
            // TODO:
        }

        let process_info = create_game_process(process_to_start_path)?;
        log::info!("Process launched (process_id: {})!", process_info.dwProcessId);

        unsafe {
            WaitForSingleObject(process_info.hProcess, INFINITE);

            // Close process and thread handles
            _ = CloseHandle(process_info.hProcess);
            _ = CloseHandle(process_info.hThread);
        }

        todo!()
    } else {
        log::info!(
            "Launching process {:?} with FFNx context: {:?}",
            process_to_start_path,
            &ctx
        );
        let process_info = create_game_process(process_to_start_path)?;
        log::info!("Process launched (process_id: {})!", process_info.dwProcessId);
        unsafe {
            WaitForSingleObject(process_info.hProcess, INFINITE);

            _ = CloseHandle(process_info.hProcess);
            _ = CloseHandle(process_info.hThread);
        }
    }

    Ok(())
}

fn create_game_process(process_to_start: PathBuf) -> Result<PROCESS_INFORMATION> {
    let startup_info = STARTUPINFOA::default();
    let mut process_info = PROCESS_INFORMATION::default();
    unsafe {
        let Ok(_) = CreateProcessA(
            PCSTR(process_to_start.into_os_string().as_encoded_bytes().as_ptr()),
            PSTR::null(),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS::default(),
            None,
            None,
            &startup_info,
            &mut process_info,
        ) else {
            _ = MessageBoxA(
                None,
                s!("Something went wrong while launching the game."),
                s!("Error"),
                MB_ICONERROR | MB_OK,
            );
            log::error!("Something went wrong while launching the game.");
            return Err(anyhow::anyhow!(
                "Something went wrong while launching the game"
            ));
        };
    }
    Ok(process_info)
}

unsafe extern "system" fn exception_handler(ep: *const EXCEPTION_POINTERS) -> i32 {
    if HAD_EXCEPTION {
        log::error!("ExceptionHandler: crash while running another Exception Handler. Exiting.");
        SetUnhandledExceptionFilter(None);
        return EXCEPTION_CONTINUE_EXECUTION;
    }

    HAD_EXCEPTION = true;
    let exception_record = &*(*ep).ExceptionRecord;
    log::error!(
        "Exception 0x{:x}, address 0x{:x}",
        exception_record.ExceptionCode.0,
        exception_record.ExceptionAddress as i32
    );
    SetUnhandledExceptionFilter(None);
    EXCEPTION_CONTINUE_EXECUTION
}
