//! Core, UI-independent logic for Refiny.
//!
//! This crate knows how to load and save configuration, build prompts, call an
//! OpenAI-compatible provider, and orchestrate one polish action. It
//! deliberately contains no platform or UI code, so it can be tested on any
//! machine.

pub mod config;
pub mod config_store;
pub mod pipeline;
pub mod presets;
pub mod prompt;
pub mod provider;
pub mod secrets;
pub mod selection;

#[cfg(test)]
mod test_support;

pub use config::{Config, ConfigError, PromptPreset, ProviderConfig};
pub use config_store::{ConfigStore, FileConfigStore, LiveConfig, MemoryConfigStore};
pub use pipeline::{ActionError, PolishAction, PolishProvider, State, StateEvent};
pub use presets::ProviderPreset;
pub use provider::{ConfiguredProvider, OpenAiCompatible, Polisher, ProviderError};
pub use secrets::{KeySource, MemorySecrets, ResolvedKey, SecretError, SecretStore};
pub use selection::{MAX_CHARS, Selection, SelectionError};
