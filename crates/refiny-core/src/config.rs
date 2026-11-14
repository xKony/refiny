//! User configuration, stored as TOML.
//!
//! Serialization order matters: top-level keys come before tables, so the
//! `provider` table and the `prompts` array of tables are the last fields.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_HOTKEY: &str = "CommandOrControl+Alt+P";
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
pub const DEFAULT_API_KEY_ENV: &str = "REFINY_API_KEY";
pub const DEFAULT_TIMEOUT_MS: u64 = 20_000;
pub const DEFAULT_MASCOT: &str = "corgi";

/// A named system prompt the user can switch to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PromptPreset {
    pub name: String,
    pub prompt: String,
}

/// Everything Refiny reads from `config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub hotkey: String,
    /// The built-in prompt template, used when no preset is active.
    pub prompt: String,
    pub preserve: Vec<String>,
    pub timeout_ms: u64,
    pub mascot: String,
    /// Name of the active entry in [`Config::prompts`]. Empty means [`Config::prompt`].
    pub active_prompt: String,
    /// True once the user has saved settings at least once.
    ///
    /// Setup is complete when configuration was saved, not when a file exists:
    /// every install creates the file, and an empty form proves nothing.
    pub setup_complete: bool,
    pub provider: ProviderConfig,
    pub prompts: Vec<PromptPreset>,
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
            mascot: DEFAULT_MASCOT.to_string(),
            active_prompt: String::new(),
            setup_complete: false,
            provider: ProviderConfig::default(),
            prompts: Vec::new(),
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

    /// Writes the config, replacing the file in one step.
    ///
    /// The text goes to a sibling temporary file that is then renamed over the
    /// target, so an interrupted write cannot leave a truncated config behind.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        self.validate()?;
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)?;

        let temporary = temporary_path(path);
        {
            let mut file = fs::File::create(&temporary)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }

        match fs::rename(&temporary, path) {
            Ok(()) => Ok(()),
            Err(rename_error) => {
                let _ = fs::remove_file(&temporary);
                // Windows refuses a rename onto a file another process holds
                // open. A direct write keeps the user from losing the save.
                if path.exists() {
                    fs::write(path, text)?;
                    Ok(())
                } else {
                    Err(ConfigError::Io(rename_error))
                }
            }
        }
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

        let mut seen: Vec<&str> = Vec::with_capacity(self.prompts.len());
        for preset in &self.prompts {
            let name = preset.name.trim();
            if name.is_empty() {
                return Err(ConfigError::Invalid(
                    "a prompt preset has an empty name".into(),
                ));
            }
            if seen.contains(&name) {
                return Err(ConfigError::Invalid(format!(
                    "duplicate prompt preset name `{name}`"
                )));
            }
            seen.push(name);
        }
        Ok(())
    }

    /// The prompt template to use: the active preset's, or the built-in one.
    ///
    /// An empty or unknown active name is not an error. A preset can be deleted
    /// while it is active, and the app still has to polish.
    pub fn prompt_template(&self) -> &str {
        let active = self.active_prompt.trim();
        if !active.is_empty()
            && let Some(preset) = self
                .prompts
                .iter()
                .find(|preset| preset.name.trim() == active)
        {
            return &preset.prompt;
        }
        &self.prompt
    }

    /// The name of the environment variable holding the API key, if any.
    pub fn api_key_env_name(&self) -> Option<&str> {
        let name = self.provider.api_key_env.trim();
        if name.is_empty() { None } else { Some(name) }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.to_path_buf();
    temporary.set_extension("toml.tmp");
    temporary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::temp_dir;

    #[test]
    fn default_config_round_trips_through_toml() {
        let config = Config::default();
        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.hotkey, DEFAULT_HOTKEY);
        assert_eq!(parsed.provider.model, DEFAULT_MODEL);
    }

    #[test]
    fn presets_round_trip_through_toml() {
        let config = Config {
            provider: ProviderConfig {
                base_url: "http://localhost:11434/v1".into(),
                ..Default::default()
            },
            prompts: vec![
                PromptPreset {
                    name: "Academic".into(),
                    prompt: "Formal.".into(),
                },
                PromptPreset {
                    name: "Casual".into(),
                    prompt: "Relaxed.".into(),
                },
            ],
            active_prompt: "Casual".into(),
            ..Default::default()
        };

        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();

        assert_eq!(parsed.prompts, config.prompts);
        assert_eq!(parsed.active_prompt, "Casual");
        assert_eq!(parsed.provider.base_url, "http://localhost:11434/v1");
    }

    #[test]
    fn load_or_create_writes_defaults() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");

        let config = Config::load_or_create(&path).unwrap();

        assert!(path.exists());
        assert_eq!(config.hotkey, DEFAULT_HOTKEY);
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("[provider]"));
    }

    #[test]
    fn save_replaces_an_existing_file() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");

        let mut config = Config::load_or_create(&path).unwrap();
        config.provider.model = "llama3.2".into();
        config.save(&path).unwrap();

        assert_eq!(Config::load(&path).unwrap().provider.model, "llama3.2");
        assert!(!dir.path().join("config.toml.tmp").exists());
    }

    #[test]
    fn save_refuses_to_write_an_invalid_config() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");
        Config::load_or_create(&path).unwrap();

        let mut config = Config::load(&path).unwrap();
        config.provider.model = String::new();

        assert!(matches!(config.save(&path), Err(ConfigError::Invalid(_))));
        assert_eq!(Config::load(&path).unwrap().provider.model, DEFAULT_MODEL);
    }

    #[test]
    fn rejects_empty_model() {
        let dir = temp_dir();
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

    #[test]
    fn rejects_duplicate_preset_names() {
        let config = Config {
            prompts: vec![
                PromptPreset {
                    name: "Same".into(),
                    prompt: "a".into(),
                },
                PromptPreset {
                    name: "Same".into(),
                    prompt: "b".into(),
                },
            ],
            ..Default::default()
        };

        let error = config.validate().unwrap_err();

        assert!(error.to_string().contains("duplicate prompt preset"));
    }

    #[test]
    fn unknown_active_preset_falls_back_to_the_configured_prompt() {
        let mut config = Config {
            prompt: "Fallback.".into(),
            active_prompt: "Missing".into(),
            ..Default::default()
        };

        assert_eq!(config.prompt_template(), "Fallback.");

        config.active_prompt = String::new();
        assert_eq!(config.prompt_template(), "Fallback.");

        config.prompts = vec![PromptPreset {
            name: "Used".into(),
            prompt: "Preset.".into(),
        }];
        config.active_prompt = "Used".into();
        assert_eq!(config.prompt_template(), "Preset.");
    }

    #[test]
    fn absent_optional_fields_load_with_defaults() {
        let dir = temp_dir();
        let path = dir.path().join("config.toml");
        // A file written by 0.0.1 has no prompts or active_prompt keys.
        fs::write(
            &path,
            r#"
hotkey = "CommandOrControl+Alt+P"

[provider]
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key_env = "REFINY_API_KEY"
"#,
        )
        .unwrap();

        let config = Config::load(&path).unwrap();

        assert!(config.prompts.is_empty());
        assert!(config.active_prompt.is_empty());
        assert_eq!(config.prompt_template(), "");
    }
}
