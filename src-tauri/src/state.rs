//! Process state: the live configuration, the secret store, and the action.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use refiny_core::{
    ConfigStore, FileConfigStore, LiveConfig, PolishAction, PolishProvider, SecretStore,
    Selection as SelectionSeam,
};
use tauri::Manager;
use tauri_plugin_global_shortcut::Shortcut;

use crate::keychain::KeychainSecrets;
use crate::selection::OsSelection;

pub struct AppState {
    /// The configuration in effect. Saves are visible to the next action.
    pub config: Arc<LiveConfig>,
    pub secrets: Arc<dyn SecretStore>,
    pub action: Arc<PolishAction>,
    /// The shortcut currently registered with the OS.
    pub hotkey: Mutex<Shortcut>,
    /// True when `config.toml` was created during this launch.
    pub first_run: bool,
}

impl AppState {
    pub fn new(
        config: Arc<LiveConfig>,
        secrets: Arc<dyn SecretStore>,
        action: Arc<PolishAction>,
        hotkey: Shortcut,
        first_run: bool,
    ) -> Self {
        Self {
            config,
            secrets,
            action,
            hotkey: Mutex::new(hotkey),
            first_run,
        }
    }

    pub fn config_path(&self) -> PathBuf {
        self.config.path().to_path_buf()
    }
}

/// Where the app writes `config.toml`, under the OS config directory.
///
/// The location follows Tauri's `app_config_dir`, so `appDirectoriesOverride`
/// in `tauri.conf.json` moves it for a portable build.
pub fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("config.toml"))
}

/// Builds the process state, creating a default config on first run.
pub fn load(app: &tauri::AppHandle) -> Result<(Arc<LiveConfig>, bool), Box<dyn std::error::Error>> {
    let path = config_path(app)?;
    let first_run = !path.exists();
    let store: Arc<dyn ConfigStore> = Arc::new(FileConfigStore::new(path));
    let config = Arc::new(LiveConfig::load(store)?);
    Ok((config, first_run))
}

/// Assembles the action from the seams the app uses in production.
pub fn build_action(config: Arc<LiveConfig>, secrets: Arc<dyn SecretStore>) -> Arc<PolishAction> {
    let selection: Arc<dyn SelectionSeam> = Arc::new(OsSelection);
    let provider: Arc<dyn PolishProvider> =
        Arc::new(refiny_core::ConfiguredProvider::new(secrets.clone()));
    Arc::new(PolishAction::new(config, selection, provider))
}

/// Creates the real keychain-backed secret store.
pub fn secrets() -> Arc<dyn SecretStore> {
    Arc::new(KeychainSecrets::new())
}
