//! Where the API key comes from.
//!
//! Resolution order is decided here, once: a stored key first, then the
//! environment variable named by `provider.api_key_env`. Nothing else in the
//! app is allowed to reach for a key another way.

use thiserror::Error;

use crate::config::Config;

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("could not reach the OS keychain: {0}")]
    Unavailable(String),
    #[error("could not read the stored key: {0}")]
    Read(String),
    #[error("could not save the key: {0}")]
    Write(String),
    #[error("could not remove the key: {0}")]
    Clear(String),
}

/// Stores one API key outside `config.toml`.
///
/// The OS keychain does not exist in CI, so tests substitute an in-memory
/// implementation.
pub trait SecretStore: Send + Sync {
    /// The stored key, or `None` when nothing is stored.
    fn get(&self) -> Result<Option<String>, SecretError>;
    fn set(&self, key: &str) -> Result<(), SecretError>;
    fn clear(&self) -> Result<(), SecretError>;
}

/// Where the key in effect came from, for display in the settings window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Keychain,
    Environment,
    None,
}

impl KeySource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keychain => "keychain",
            Self::Environment => "environment",
            Self::None => "none",
        }
    }
}

/// The key in effect, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedKey {
    pub key: Option<String>,
    pub source: KeySource,
}

/// The key to use for a provider call.
///
/// A keychain that is unavailable is not an error on its own: a user who only
/// ever set an environment variable must keep working on a machine with no
/// Secret Service running. It is only an error when asking to store a key.
pub fn resolve(config: &Config, secrets: &dyn SecretStore) -> ResolvedKey {
    if let Ok(Some(key)) = secrets.get()
        && !key.is_empty()
    {
        return ResolvedKey {
            key: Some(key),
            source: KeySource::Keychain,
        };
    }

    if let Some(name) = config.api_key_env_name()
        && let Ok(value) = std::env::var(name)
        && !value.is_empty()
    {
        return ResolvedKey {
            key: Some(value),
            source: KeySource::Environment,
        };
    }

    ResolvedKey {
        key: None,
        source: KeySource::None,
    }
}

/// A [`SecretStore`] that keeps the key in memory.
///
/// The OS keychain does not exist in CI, so tests - and any test path in the
/// app - substitute this instead.
pub struct MemorySecrets {
    key: std::sync::Mutex<Option<String>>,
    available: std::sync::Mutex<bool>,
}

impl MemorySecrets {
    pub fn with_key(key: &str) -> Self {
        Self {
            key: std::sync::Mutex::new(Some(key.to_string())),
            available: std::sync::Mutex::new(true),
        }
    }

    /// A store that behaves like a machine with no Secret Service running.
    pub fn unavailable() -> Self {
        Self {
            key: std::sync::Mutex::new(None),
            available: std::sync::Mutex::new(false),
        }
    }

    pub fn stored(&self) -> Option<String> {
        self.key.lock().unwrap().clone()
    }
}

impl Default for MemorySecrets {
    fn default() -> Self {
        Self {
            key: std::sync::Mutex::new(None),
            available: std::sync::Mutex::new(true),
        }
    }
}

impl SecretStore for MemorySecrets {
    fn get(&self) -> Result<Option<String>, SecretError> {
        if !*self.available.lock().unwrap() {
            return Err(SecretError::Unavailable("no keychain".into()));
        }
        Ok(self.key.lock().unwrap().clone())
    }

    fn set(&self, key: &str) -> Result<(), SecretError> {
        if !*self.available.lock().unwrap() {
            return Err(SecretError::Unavailable("no keychain".into()));
        }
        *self.key.lock().unwrap() = Some(key.to_string());
        Ok(())
    }

    fn clear(&self) -> Result<(), SecretError> {
        if !*self.available.lock().unwrap() {
            return Err(SecretError::Unavailable("no keychain".into()));
        }
        *self.key.lock().unwrap() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_env(name: &str) -> Config {
        Config {
            provider: crate::config::ProviderConfig {
                api_key_env: name.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn a_key_can_be_stored_read_and_cleared() {
        let secrets = MemorySecrets::default();

        assert_eq!(secrets.get().unwrap(), None);
        secrets.set("sk-secret").unwrap();
        assert_eq!(secrets.get().unwrap().as_deref(), Some("sk-secret"));
        secrets.clear().unwrap();
        assert_eq!(secrets.get().unwrap(), None);
        assert_eq!(secrets.stored(), None);
    }

    #[test]
    fn an_unavailable_keychain_reports_itself() {
        let secrets = MemorySecrets::unavailable();

        let error = secrets.set("sk-secret").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("could not reach the OS keychain")
        );
        assert_eq!(secrets.stored(), None);
    }

    #[test]
    fn a_stored_key_wins_over_the_environment() {
        let name = "REFINY_TEST_KEY_KEYCHAIN";
        // SAFETY: this test owns the variable and no other test reads it.
        unsafe { std::env::set_var(name, "from-env") };
        let config = config_with_env(name);
        let secrets = MemorySecrets::with_key("from-keychain");

        let resolved = resolve(&config, &secrets);

        assert_eq!(resolved.key.as_deref(), Some("from-keychain"));
        assert_eq!(resolved.source, KeySource::Keychain);
        unsafe { std::env::remove_var(name) };
    }

    #[test]
    fn the_environment_is_used_when_nothing_is_stored() {
        let name = "REFINY_TEST_KEY_ENV_ONLY";
        unsafe { std::env::set_var(name, "from-env") };
        let config = config_with_env(name);
        let secrets = MemorySecrets::default();

        let resolved = resolve(&config, &secrets);

        assert_eq!(resolved.key.as_deref(), Some("from-env"));
        assert_eq!(resolved.source, KeySource::Environment);
        unsafe { std::env::remove_var(name) };
    }

    #[test]
    fn a_local_provider_without_a_key_resolves_to_none() {
        let config = config_with_env("REFINY_TEST_KEY_ABSENT");
        let secrets = MemorySecrets::default();

        let resolved = resolve(&config, &secrets);

        assert_eq!(resolved.key, None);
        assert_eq!(resolved.source, KeySource::None);
    }

    #[test]
    fn an_unavailable_keychain_still_allows_an_environment_key() {
        let name = "REFINY_TEST_KEY_FALLBACK";
        unsafe { std::env::set_var(name, "from-env") };
        let config = config_with_env(name);
        let secrets = MemorySecrets::unavailable();

        let resolved = resolve(&config, &secrets);

        assert_eq!(resolved.key.as_deref(), Some("from-env"));
        assert_eq!(resolved.source, KeySource::Environment);
        unsafe { std::env::remove_var(name) };
    }
}
