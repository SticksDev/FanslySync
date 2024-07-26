use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::structs::{FanslyFollowersResponse, Subscription};

const CURRENT_VERSION: i32 = 1; // Set the current version of the config

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
            let mut config: Self = serde_json::from_str(std::fs::read_to_string(path)?.as_str())
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Could not parse config file: {}", e),
                    )
                })?;
            if config.version != CURRENT_VERSION {
                config = config.migrate()?;
                config.save(path)?;
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
                    // If we're on version 1, migrate to version 2 (not implemented)
                    self.version += 1;
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
