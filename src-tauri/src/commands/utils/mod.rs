#[tauri::command]
pub fn quit(code: i32) {
    std::process::exit(code);
}
