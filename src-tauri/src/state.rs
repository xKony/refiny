use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use refiny_core::Config;
use tauri::Manager;

pub struct AppState {
    pub config: Config,
    pub config_path: PathBuf,
    /// True while a polish action is running; the hotkey is ignored then.
    pub busy: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
            busy: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn load_config(
    app: &tauri::AppHandle,
) -> Result<(Config, PathBuf), Box<dyn std::error::Error>> {
    let dir = app.path().app_config_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join("config.toml");
    let config = Config::load_or_create(&path)?;
    Ok((config, path))
}
