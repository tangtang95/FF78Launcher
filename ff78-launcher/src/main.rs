#![windows_subsystem = "windows"]

use anyhow::Result;
use config::Config;
use launcher::{write_ffsound, write_ffvideo};
use log::LevelFilter;
use std::{ffi::CString, os::windows::fs::MetadataExt};
use windows::{
    core::{s, PCSTR, PSTR},
    Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::Debug::{
                SetUnhandledExceptionFilter, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_POINTERS,
            },
            Memory::{
                CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
                PAGE_READWRITE,
            },
            Threading::{
                CreateProcessA, CreateSemaphoreA, CreateThread, WaitForSingleObject, INFINITE,
                PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOA, THREAD_CREATION_FLAGS,
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

unsafe extern "system" fn process_game_messages(_: *mut core::ffi::c_void) -> u32 {
    todo!()
}

fn main() -> Result<()> {
    simple_logging::log_to_file(LOG_FILE, LevelFilter::Info)?;
    log::info!("{APP_NAME} launched!");

    unsafe {
        SetUnhandledExceptionFilter(Some(exception_handler));
    };

    match launch_process() {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Launching process failed due: {:?}", err);
            Err(err)
        }
    }
}

fn launch_process() -> Result<()> {
    let processes_available: Vec<&str> = PROCESSES
        .into_iter()
        .filter(|process| matches!(std::fs::exists(process), Ok(true)))
        .collect();
    if processes_available.len() > 1 {
        return Err(anyhow::anyhow!(
            "More than one process to start found: {:?}",
            processes_available
        ));
    }
    let Some(mut process_to_start) = processes_available.first().map(|s| s.to_string()) else {
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
        return Err(anyhow::anyhow!(
            "No language found for process: {}",
            process_to_start
        ));
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

    let process_filename = std::fs::canonicalize(&process_to_start)?
        .file_name()
        .ok_or(anyhow::anyhow!("Filename of process not found"))?
        .to_os_string()
        .into_string()
        .map_err(|_| anyhow::anyhow!("Couldn't transform OsString to String"))?;
    if !use_ffnx || ctx.config.launch_chocobo {
        log::info!(
            "Launching process {:?} without FFNx context: {:?}",
            process_filename,
            &ctx
        );
        if !use_ffnx {
            write_ffvideo(&ctx)?;
            write_ffsound(&ctx)?;
        }
        let name_prefix = match ctx.config.launch_chocobo {
            true => "choco",
            false => match ctx.game_to_launch {
                GameType::FF7(_) => "ff7",
                GameType::FF8 => "ff8",
            },
        };
        let game_can_read_name = CString::new(name_prefix.to_owned() + "_gameCanReadMsgSem")?;
        let game_did_read_name = CString::new(name_prefix.to_owned() + "_gameDidReadMsgSem")?;
        let launcher_can_read_name =
            CString::new(name_prefix.to_owned() + "_launcherCanReadMsgSem")?;
        let launcher_did_read_name =
            CString::new(name_prefix.to_owned() + "_launcherDidReadMsgSem")?;
        let shared_memory_name =
            CString::new(name_prefix.to_owned() + "_sharedMemoryWithLauncher")?;
        unsafe {
            let game_can_read_sem =
                CreateSemaphoreA(None, 0, 1, PCSTR(game_can_read_name.as_ptr() as _))?;
            let game_did_read_sem =
                CreateSemaphoreA(None, 0, 1, PCSTR(game_did_read_name.as_ptr() as _))?;
            let launcher_can_read_sem =
                CreateSemaphoreA(None, 0, 1, PCSTR(launcher_can_read_name.as_ptr() as _))?;
            let launcher_did_read_sem =
                CreateSemaphoreA(None, 0, 1, PCSTR(launcher_did_read_name.as_ptr() as _))?;
            let shared_memory = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                0x20000,
                PCSTR(shared_memory_name.as_ptr() as _),
            )?;
            let view_shared_memory = MapViewOfFile(shared_memory, FILE_MAP_ALL_ACCESS, 0, 0, 0);
            let launcher_memory_part = view_shared_memory.Value.offset(0x10000);
            let process_game_messages_thread = CreateThread(
                None,
                0,
                Some(process_game_messages),
                None,
                THREAD_CREATION_FLAGS::default(),
                None,
            )?;
            let process_info = create_game_process(CString::new(process_filename)?)?;
            log::info!(
                "Process launched (process_id: {})!",
                process_info.dwProcessId
            );

            // send_locale_data_dir();
            // send_user_save_dir();
            // send_user_doc_dir();
            // send_install_dir();
            // send_game_version();
            // send_disable_cloud();
            // send_bg_pause_enabled();
            // send_launcher_completed();

            WaitForSingleObject(process_info.hProcess, INFINITE);

            // Close process and thread handles
            _ = CloseHandle(process_info.hProcess);
            _ = CloseHandle(process_info.hThread);
            _ = CloseHandle(process_game_messages_thread);

            _ = UnmapViewOfFile(view_shared_memory);
            _ = CloseHandle(shared_memory);
            _ = CloseHandle(game_did_read_sem);
            _ = CloseHandle(game_can_read_sem);
            _ = CloseHandle(launcher_did_read_sem);
            _ = CloseHandle(launcher_can_read_sem);
        }
    } else {
        log::info!(
            "Launching process {:?} with FFNx context: {:?}",
            process_filename,
            &ctx
        );
        let process_info = create_game_process(CString::new(process_filename)?)?;
        log::info!(
            "Process launched (process_id: {})!",
            process_info.dwProcessId
        );
        unsafe {
            WaitForSingleObject(process_info.hProcess, INFINITE);

            _ = CloseHandle(process_info.hProcess);
            _ = CloseHandle(process_info.hThread);
        }
    }

    Ok(())
}

fn create_game_process(process_to_start: CString) -> Result<PROCESS_INFORMATION> {
    let startup_info = STARTUPINFOA {
        cb: size_of::<STARTUPINFOA>() as u32,
        ..Default::default()
    };
    let mut process_info = PROCESS_INFORMATION::default();
    unsafe {
        match CreateProcessA(
            PCSTR(process_to_start.as_ptr() as _),
            PSTR::null(),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS::default(),
            None,
            None,
            &startup_info,
            &mut process_info,
        ) {
            Ok(_) => {}
            Err(err) => {
                _ = MessageBoxA(
                    None,
                    s!("Something went wrong while launching the game."),
                    s!("Error"),
                    MB_ICONERROR | MB_OK,
                );
                return Err(anyhow::anyhow!(format!(
                    "Something went wrong while launching the game: {:?}",
                    err
                )));
            }
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
