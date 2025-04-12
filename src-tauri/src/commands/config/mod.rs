use crate::handlers::config::{get_config_path, Config};

#[tauri::command]
pub fn init_config() -> Result<(), String> {
    log::info!("[commands::config::init_config] Initializing config...");
    let config_path = get_config_path().map_err(|e| e.to_string())?;

    log::info!(
        "[commands::config::init_config] Config path: {}",
        config_path.display()
    );

    Config::load_or_create(&config_path).map_err(|e| e.to_string())?;
    log::info!("[commands::config::init_config] Config initialized successfully");
    Ok(())
}

#[tauri::command]
pub fn get_config() -> Result<Config, String> {
    let config_path = get_config_path().map_err(|e| e.to_string())?;
    let config = Config::load_or_create(&config_path).map_err(|e| e.to_string())?;

    log::info!(
        "[commands::config::get_config] Config loaded successfully: {:?} from path: {}",
        config,
        config_path.display()
    );

    Ok(config)
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<(), String> {
    let config_path = get_config_path().map_err(|e| e.to_string())?;
    log::info!(
        "[commands::config::save_config] Saving config: {:?} to path: {}",
        config,
        config_path.display()
    );

    config.save(&config_path).map_err(|e| e.to_string())?;
    Ok(())
}
