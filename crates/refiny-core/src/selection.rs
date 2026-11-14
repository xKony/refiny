//! The selection seam: what a polish action needs from the operating system.

use thiserror::Error;

/// Selections longer than this are rejected before they reach the provider.
pub const MAX_CHARS: usize = 20_000;

#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("could not access the clipboard: {0}")]
    Clipboard(String),
    #[error("could not send synthetic input: {0}")]
    Input(String),
    #[error("nothing was selected")]
    NothingSelected,
    #[error("refusing to paste empty text")]
    Empty,
    #[error("the selection is longer than {0} characters")]
    TooLong(usize),
}

impl SelectionError {
    /// A message suitable for the overlay, with platform-specific hints.
    pub fn user_message(&self) -> String {
        match self {
            Self::NothingSelected if cfg!(target_os = "macos") => "Nothing was selected. If this keeps happening, grant Refiny Accessibility permission in System Settings, Privacy & Security, Accessibility.".to_string(),
            Self::NothingSelected if cfg!(target_os = "linux") => {
                "Nothing was selected. Wayland sessions may block synthetic input; X11 is required."
                    .to_string()
            }
            Self::NothingSelected => {
                "Nothing was selected. Elevated windows and some apps block synthetic input."
                    .to_string()
            }
            other => other.to_string(),
        }
    }
}

/// Reads and replaces the selection in the foreground application.
///
/// Implemented by the Tauri shell against the real OS, and by fakes in tests.
pub trait Selection: Send + Sync {
    fn capture(&self) -> Result<String, SelectionError>;
    fn replace(&self, text: &str) -> Result<(), SelectionError>;
}
