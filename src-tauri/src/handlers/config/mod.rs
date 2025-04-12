use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::structs::{FanslyFollowersResponse, Subscription};

const CURRENT_VERSION: i32 = 2; // Set the current version of the config

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncData {
    pub followers: Vec<FanslyFollowersResponse>,
    pub subscribers: Vec<Subscription>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: i32, // Add a version field to the config (1, 2, 3, etc.)
    pub is_first_run: bool,
    pub fansly_token: String,
    pub auto_sync_enabled: bool,
    pub sync_token: String,
    pub sync_interval: u64,
    pub last_sync: u64,
    pub last_sync_data: SyncData,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: CURRENT_VERSION,    // Version is set to CURRENT_VERSION by default
            is_first_run: true,          // First run is set to true by default
            fansly_token: String::new(), // Fansly token is stored as a string
            sync_interval: 1,            // Every hour - sync interval is interpreted as hours
            last_sync: 0,                // Last sync time is stored as a UNIX timestamp
            auto_sync_enabled: false,    // Auto sync is disabled by default
            sync_token: String::new(),   // Sync token is stored as a string
            last_sync_data: SyncData {
                followers: Vec::new(),
                subscribers: Vec::new(),
            }, // Last sync data is stored as a list of followers and subscribers
        }
    }
}

impl Config {
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if path.exists() {
            let config_result: Result<Self, _> =
                serde_json::from_str(&std::fs::read_to_string(path)?);
            let config = match config_result {
                Ok(config) => config,
                Err(_) => {
                    // Load raw JSON and attempt to parse it as a JSON object
                    let config_raw = std::fs::read_to_string(path)?;
                    let config_json: serde_json::Value = serde_json::from_str(&config_raw)?;

                    log::info!("[config::migrate] Migrating config file to latest version...");
                    log::debug!(
                        "[config::migrate] [DEBUG] config is_object: {}",
                        config_json.is_object()
                    );

                    // Check if the JSON object is valid, if not, return an error
                    if !config_json.is_object() {
                        log::error!(
                            "[config::migrate] [ERROR] Found invalid JSON object in config file"
                        );
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Tried to migrate a config file, but found an invalid JSON object",
                        ));
                    }

                    // Get the version field from the JSON object
                    let version = config_json["version"].as_i64().unwrap_or(0) as i32;

                    // Check if the version field is a valid integer, if not, return an error
                    if version == 0 {
                        log::error!(
                            "[config::migrate] [ERROR] Found invalid version field in config JSON"
                        );
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Tried to migrate a config file, but found an invalid version field",
                        ));
                    }

                    log::info!(
                        "[config::migrate] Found version field in config JSON: {}",
                        version
                    );

                    // Now create a new Config object and set the version field to the value we found
                    let mut config = Config::default();
                    config.version = version;

                    // Retain important fields from the JSON object
                    config.is_first_run = config_json["is_first_run"].as_bool().unwrap_or(true);
                    config.fansly_token = config_json["fansly_token"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    config.sync_token =
                        config_json["sync_token"].as_str().unwrap_or("").to_string();
                    config.sync_interval =
                        config_json["sync_interval"].as_i64().unwrap_or(1) as u64;

                    // Run migrations on the config object and save it
                    config = config.migrate()?;
                    config.save(path)?;

                    log::info!(
                        "[config::migrate] Successfully migrated config file to latest version"
                    );
                    // Recursively call load_or_create to load the migrated config
                    return Config::load_or_create(path);
                }
            };

            if config.version != CURRENT_VERSION {
                // Should have been migrated by now, error out because it wasn't
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Config version mismatch: expected {}, got {}. Please try removing the config file and restarting the application.", 
                        CURRENT_VERSION, config.version
                    ),
                ));
            }

            Ok(config)
        } else {
            let saved_config = Config::default().save(path);
            saved_config
                .and_then(|_| Config::load_or_create(path))
                .or_else(|e| Err(e))
        }
    }

    fn migrate(mut self) -> io::Result<Self> {
        while self.version < CURRENT_VERSION {
            self = match self.version {
                1 => {
                    // Migrate from version 1 to version 2
                    self.version = 2;
                    self.auto_sync_enabled = false;
                    self.sync_token = String::new();
                    self.sync_interval = 1;

                    self
                }
                _ => {
                    // If we don't have a migration path, return an error
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("No migration path for version {}", self.version),
                    ));
                }
            };
        }
        Ok(self)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(serde_json::to_string_pretty(self).unwrap().as_bytes())?;

        // Return the saved config
        Ok(())
    }
}

pub fn get_config_path() -> io::Result<PathBuf> {
    let mut config_dir = dirs::config_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine user's config directory",
        )
    })?;
    config_dir.push("FanslySync");
    fs::create_dir_all(&config_dir)?;
    config_dir.push("config.json");
    Ok(config_dir)
}
