//! Core, UI-independent logic for Refiny.
//!
//! This crate knows how to load configuration, build prompts, and call an
//! OpenAI-compatible provider. It deliberately contains no platform or UI code
//! so it can be tested on any machine.

pub mod config;
pub mod prompt;
pub mod provider;

pub use config::Config;
pub use provider::{OpenAiCompatible, Polisher, ProviderError};
