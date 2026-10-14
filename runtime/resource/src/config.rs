use std::path::PathBuf;

use serde::Deserialize;

use platform::{file_service, path};

const CONFIG_FILE_NAME: &str = "engine.toml";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    pub window: WindowSettings,
    pub log_level: String,
    pub vsync: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window: WindowSettings::default(),
            log_level: "info".to_string(),
            vsync: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WindowSettings {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            title: "CuEngine".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

pub struct ConfigManager {
    config: EngineConfig,
}

impl ConfigManager {
    pub fn load() -> Self {
        Self {
            config: load_config(),
        }
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

fn config_path() -> Option<PathBuf> {
    let cwd_candidate = PathBuf::from(CONFIG_FILE_NAME);
    if file_service::exists(&cwd_candidate) {
        return Some(cwd_candidate);
    }

    let exe_candidate = path::join(&path::executable_dir(), CONFIG_FILE_NAME);
    if file_service::exists(&exe_candidate) {
        return Some(exe_candidate);
    }

    None
}

fn load_config() -> EngineConfig {
    let Some(config_path) = config_path() else {
        engine_core::log_warn!("config `{CONFIG_FILE_NAME}` not found, using defaults");
        return EngineConfig::default();
    };

    match file_service::read_to_string(&config_path) {
        Ok(text) => match toml::from_str::<EngineConfig>(&text) {
            Ok(config) => {
                engine_core::log_info!("config loaded from {}", config_path.display());
                config
            }
            Err(error) => {
                engine_core::log_error!("failed to parse {}: {error}", config_path.display());
                EngineConfig::default()
            }
        },
        Err(error) => {
            engine_core::log_error!("failed to read {}: {error}", config_path.display());
            EngineConfig::default()
        }
    }
}
