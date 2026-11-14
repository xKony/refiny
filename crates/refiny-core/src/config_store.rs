//! Where the configuration lives, and the live copy the app reads from.
//!
//! The app never holds a `Config` value captured at startup. It holds a
//! [`LiveConfig`], which owns a [`ConfigStore`] and the current value, so a save
//! is visible to the next polish action without a restart.

use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

use crate::config::{Config, ConfigError};

/// Storage for `config.toml`.
///
/// The seam exists so tests can substitute a store that never touches the real
/// user profile.
pub trait ConfigStore: Send + Sync {
    /// Reads the stored config, creating a default file when none exists.
    fn load(&self) -> Result<Config, ConfigError>;
    fn save(&self, config: &Config) -> Result<(), ConfigError>;
    fn path(&self) -> &Path;
}

/// A [`ConfigStore`] backed by a file on disk.
#[derive(Debug, Clone)]
pub struct FileConfigStore {
    path: PathBuf,
}

impl FileConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl ConfigStore for FileConfigStore {
    fn load(&self) -> Result<Config, ConfigError> {
        Config::load_or_create(&self.path)
    }

    fn save(&self, config: &Config) -> Result<(), ConfigError> {
        config.save(&self.path)
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

/// The configuration currently in effect.
///
/// Reads are cheap and lock-scoped, so a caller can take a snapshot, release the
/// lock, and make a provider call without holding anything else off.
pub struct LiveConfig {
    store: Arc<dyn ConfigStore>,
    current: RwLock<Config>,
}

impl LiveConfig {
    pub fn new(store: Arc<dyn ConfigStore>, config: Config) -> Self {
        Self {
            store,
            current: RwLock::new(config),
        }
    }

    /// Loads from `store` and returns a live config holding the result.
    pub fn load(store: Arc<dyn ConfigStore>) -> Result<Self, ConfigError> {
        let config = store.load()?;
        Ok(Self::new(store, config))
    }

    /// The configuration in effect right now.
    pub fn snapshot(&self) -> Config {
        self.current
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn path(&self) -> &Path {
        self.store.path()
    }

    /// Re-reads the stored config, discarding unsaved changes.
    pub fn reload(&self) -> Result<Config, ConfigError> {
        let config = self.store.load()?;
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = config.clone();
        Ok(config)
    }

    /// Persists `next` and only then makes it current.
    ///
    /// A write that fails leaves the previous configuration in effect, so the
    /// app keeps working on values that match the file.
    pub fn save(&self, next: Config) -> Result<Config, ConfigError> {
        self.store.save(&next)?;
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = next.clone();
        Ok(next)
    }

    /// Applies `edit` to a copy of the current config and saves the result.
    ///
    /// The edit runs against a copy: a rejected value never reaches the file or
    /// the running app.
    pub fn update(&self, edit: impl FnOnce(&mut Config)) -> Result<Config, ConfigError> {
        let mut next = self.snapshot();
        edit(&mut next);
        self.save(next)
    }
}

/// A [`ConfigStore`] that keeps its config in memory.
///
/// Used by tests, and by the app's own test path, so nothing there touches the
/// real user profile.
pub struct MemoryConfigStore {
    path: PathBuf,
    config: std::sync::Mutex<Option<Config>>,
    saves: std::sync::Mutex<Vec<Config>>,
}

impl MemoryConfigStore {
    pub fn new() -> Self {
        Self {
            path: PathBuf::from("memory/config.toml"),
            config: std::sync::Mutex::new(None),
            saves: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_config(config: Config) -> Self {
        let store = Self::new();
        *store.config.lock().unwrap() = Some(config);
        store
    }

    pub fn save_count(&self) -> usize {
        self.saves.lock().unwrap().len()
    }

    /// Writes without going through the store, as a hand-edited file would.
    pub fn write_externally(&self, config: Config) {
        *self.config.lock().unwrap() = Some(config);
    }
}

impl Default for MemoryConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigStore for MemoryConfigStore {
    fn load(&self) -> Result<Config, ConfigError> {
        Ok(self.config.lock().unwrap().clone().unwrap_or_default())
    }

    fn save(&self, config: &Config) -> Result<(), ConfigError> {
        config.validate()?;
        *self.config.lock().unwrap() = Some(config.clone());
        self.saves.lock().unwrap().push(config.clone());
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DEFAULT_MODEL, PromptPreset};
    use crate::test_support::temp_dir;

    fn live(store: Arc<MemoryConfigStore>) -> LiveConfig {
        LiveConfig::load(store).unwrap()
    }

    #[test]
    fn a_save_is_visible_to_the_next_read() {
        let store = Arc::new(MemoryConfigStore::new());
        let config = live(store.clone());

        assert_eq!(config.snapshot().provider.model, DEFAULT_MODEL);

        config
            .update(|next| next.provider.model = "llama3.2".into())
            .unwrap();

        assert_eq!(config.snapshot().provider.model, "llama3.2");
        assert_eq!(store.save_count(), 1);
        assert_eq!(store.load().unwrap().provider.model, "llama3.2");
    }

    #[test]
    fn a_rejected_save_changes_nothing() {
        let store = Arc::new(MemoryConfigStore::new());
        let config = live(store.clone());

        let error = config
            .update(|next| next.provider.model = String::new())
            .unwrap_err();

        assert!(error.to_string().contains("provider.model"));
        assert_eq!(config.snapshot().provider.model, DEFAULT_MODEL);
        assert_eq!(store.save_count(), 0);
    }

    #[test]
    fn reload_picks_up_a_hand_edited_file() {
        let store = Arc::new(MemoryConfigStore::new());
        let config = live(store.clone());

        let mut edited = Config::default();
        edited.provider.model = "hand-edited".into();
        store.write_externally(edited);

        assert_eq!(config.snapshot().provider.model, DEFAULT_MODEL);
        assert_eq!(config.reload().unwrap().provider.model, "hand-edited");
        assert_eq!(config.snapshot().provider.model, "hand-edited");
    }

    #[test]
    fn a_corrupt_file_is_reported_and_nothing_is_lost() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "hotkey = [not toml").unwrap();

        let error = match LiveConfig::load(Arc::new(FileConfigStore::new(&path))) {
            Ok(_) => panic!("a corrupt config must not load"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("could not parse config"));
    }

    #[test]
    fn a_saved_value_survives_a_restart() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");
        let store = Arc::new(FileConfigStore::new(&path));

        let config = LiveConfig::load(store.clone()).unwrap();
        config
            .update(|next| next.provider.model = "llama3.2".into())
            .unwrap();

        let restarted = LiveConfig::load(store).unwrap();
        assert_eq!(restarted.snapshot().provider.model, "llama3.2");
    }

    #[test]
    fn switching_the_active_preset_changes_the_template() {
        let store = Arc::new(MemoryConfigStore::new());
        let config = live(store);
        config
            .update(|next| {
                next.prompt = "Base.".into();
                next.prompts = vec![
                    PromptPreset {
                        name: "Terse".into(),
                        prompt: "Be terse.".into(),
                    },
                    PromptPreset {
                        name: "Warm".into(),
                        prompt: "Be warm.".into(),
                    },
                ];
            })
            .unwrap();

        config
            .update(|next| next.active_prompt = "Warm".into())
            .unwrap();

        assert_eq!(config.snapshot().prompt_template(), "Be warm.");

        config
            .update(|next| next.active_prompt = "Gone".into())
            .unwrap();
        assert_eq!(config.snapshot().prompt_template(), "Base.");
    }
}
