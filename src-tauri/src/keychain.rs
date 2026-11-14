//! The real OS credential store, behind the core `SecretStore` seam.
//!
//! Windows uses Credential Manager, macOS uses Keychain Services, and Linux
//! uses the Secret Service over D-Bus. None of these exist in CI, which is why
//! the app talks to [`refiny_core::SecretStore`] and tests swap in
//! [`refiny_core::MemorySecrets`].

use refiny_core::{SecretError, SecretStore};

const SERVICE: &str = "dev.refiny.app";
const ACCOUNT: &str = "api-key";

/// The platform keychain entry holding the API key.
pub struct KeychainSecrets {
    service: String,
    account: String,
}

impl KeychainSecrets {
    pub fn new() -> Self {
        Self {
            service: SERVICE.to_string(),
            account: ACCOUNT.to_string(),
        }
    }

    fn entry(&self) -> Result<keyring::Entry, SecretError> {
        // `store_status` performs the one-time backend initialization, so a
        // missing Secret Service is reported here rather than as a confusing
        // per-operation failure.
        if let Err(error) = keyring::Entry::store_status() {
            return Err(SecretError::Unavailable(error.to_string()));
        }
        keyring::Entry::new(&self.service, &self.account).map_err(unavailable)
    }
}

impl Default for KeychainSecrets {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for KeychainSecrets {
    fn get(&self) -> Result<Option<String>, SecretError> {
        match self.entry()?.get_password() {
            Ok(key) => Ok(Some(key)),
            // A missing entry is the normal state for a user who has never
            // saved a key, not an error they should see.
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(SecretError::Read(error.to_string())),
        }
    }

    fn set(&self, key: &str) -> Result<(), SecretError> {
        self.entry()?
            .set_password(key)
            .map_err(|error| SecretError::Write(error.to_string()))
    }

    fn clear(&self) -> Result<(), SecretError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(SecretError::Clear(error.to_string())),
        }
    }
}

fn unavailable(error: keyring::Error) -> SecretError {
    SecretError::Unavailable(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the real keychain on the machine running the tests, and is
    /// ignored by default so CI never depends on a credential store.
    #[test]
    #[ignore = "requires a real OS credential store"]
    fn round_trips_a_key_through_the_real_keychain() {
        let secrets = KeychainSecrets {
            service: format!("{SERVICE}-test"),
            account: "round-trip".into(),
        };
        secrets.set("sk-test-value").unwrap();
        assert_eq!(secrets.get().unwrap().as_deref(), Some("sk-test-value"));
        secrets.clear().unwrap();
        assert_eq!(secrets.get().unwrap(), None);
    }
}
