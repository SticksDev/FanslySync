// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod handlers;
mod structs;

use commands::config::{get_config, init_config, save_config};
use commands::fansly::{fansly_get_me, fansly_set_token, fansly_sync};
use commands::utils::quit;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
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
