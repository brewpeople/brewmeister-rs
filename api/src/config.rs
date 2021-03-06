use crate::Result;
use serde::Deserialize;
use std::path::PathBuf;

const DEFAULT_DEVICE_PATH: &str = "/dev/ttyACM0";

/// Server configuration.
pub struct Config {
    /// Path to the brewslave device. By default this is /dev/ttyACM0.
    pub device: PathBuf,
    /// Path to the database file or `None`.
    pub database: Option<String>,
}

#[derive(Deserialize)]
struct Serialized {
    #[serde(default)]
    device: Option<PathBuf>,
    #[serde(default)]
    database: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: PathBuf::from(DEFAULT_DEVICE_PATH),
            database: None,
        }
    }
}

impl Config {
    /// Read from brewmeister.toml or return some defaults.
    pub fn new() -> Result<Self> {
        let path = PathBuf::from("brewmeister.toml");

        if path.exists() && path.is_file() {
            let content = std::fs::read_to_string(path)?;
            let config: Serialized = toml::from_str(&content)?;

            Ok(Self {
                device: config
                    .device
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_DEVICE_PATH)),
                database: config.database,
            })
        } else {
            Ok(Self::default())
        }
    }
}
