//! User configuration, stored as TOML.
//!
//! The `provider` table is deliberately the last field in [`Config`]: TOML
//! requires table headers to come after top-level keys, and serde serializes
//! fields in declaration order.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_HOTKEY: &str = "CommandOrControl+Alt+P";
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_API_KEY_ENV: &str = "REFINY_API_KEY";
pub const DEFAULT_TIMEOUT_MS: u64 = 20_000;

/// Everything Refiny reads from `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hotkey: String,
    pub prompt: String,
    pub preserve: Vec<String>,
    pub timeout_ms: u64,
    pub mascot: String,
    pub provider: ProviderConfig,
}

/// An OpenAI-compatible endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: DEFAULT_HOTKEY.to_string(),
            prompt: String::new(),
            preserve: Vec::new(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            mascot: "corgi".to_string(),
            provider: ProviderConfig::default(),
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            api_key_env: DEFAULT_API_KEY_ENV.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not access config: {0}")]
    Io(#[from] io::Error),
    #[error("could not parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("could not serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid config: {0}")]
    Invalid(String),
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&text)?;
        config.validate()?;
        Ok(config)
    }

    /// Loads `path`, creating it with defaults when it does not exist.
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if path.exists() {
            return Self::load(path);
        }
        let config = Config::default();
        config.save(path)?;
        Ok(config)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.hotkey.trim().is_empty() {
            return Err(ConfigError::Invalid("hotkey must not be empty".into()));
        }
        if self.provider.base_url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "provider.base_url must not be empty".into(),
            ));
        }
        if self.provider.model.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "provider.model must not be empty".into(),
            ));
        }
        if self.timeout_ms == 0 {
            return Err(ConfigError::Invalid(
                "timeout_ms must be greater than zero".into(),
            ));
        }
        Ok(())
    }

    /// Reads the API key from the environment variable named in the config.
    pub fn api_key(&self) -> Option<String> {
        let name = self.provider.api_key_env.trim();
        if name.is_empty() {
            return None;
        }
        std::env::var(name).ok().filter(|value| !value.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_round_trips_through_toml() {
        let config = Config::default();
        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.hotkey, DEFAULT_HOTKEY);
        assert_eq!(parsed.provider.model, DEFAULT_MODEL);
    }

    #[test]
    fn load_or_create_writes_defaults() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = Config::load_or_create(&path).unwrap();

        assert!(path.exists());
        assert_eq!(config.hotkey, DEFAULT_HOTKEY);
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("[provider]"));
    }

    #[test]
    fn rejects_empty_model() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let text = r#"
hotkey = "CommandOrControl+Alt+P"

[provider]
base_url = "http://localhost:11434/v1"
model = ""
"#;
        fs::write(&path, text).unwrap();

        let error = Config::load(&path).unwrap_err();

        assert!(error.to_string().contains("provider.model"));
    }
}
