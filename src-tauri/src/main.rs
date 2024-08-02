// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod handlers;
mod structs;

use std::fs;
use std::io;

use commands::config::{get_config, init_config, save_config};
use commands::fansly::{fansly_get_me, fansly_set_token, fansly_sync};
use commands::utils::quit;
use tauri_plugin_log::{Target, TargetKind};

fn get_log_path() -> io::Result<String> {
    let mut config_dir = dirs::config_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine user's config directory",
        )
    })?;
    config_dir.push("FanslySync");
    fs::create_dir_all(&config_dir)?;
    config_dir.push("runtime");

    // Return the path as a string
    Ok(config_dir.to_string_lossy().to_string())
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(get_log_path().unwrap()),
                    }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            init_config,
            get_config,
            save_config,
            quit,
            fansly_set_token,
            fansly_get_me,
            fansly_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
