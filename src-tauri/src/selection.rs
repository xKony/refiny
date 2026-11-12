//! Platform boundary for reading and replacing the foreground selection.
//!
//! The clipboard round-trip is deliberately isolated here so the rest of the
//! app never depends on OS APIs. The first implementation will:
//!
//! 1. snapshot the clipboard,
//! 2. send the platform copy shortcut,
//! 3. read the clipboard,
//! 4. restore the snapshot.
//!
//! Replacement is the same flow with the paste shortcut.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("selection capture is not implemented yet")]
    NotImplemented,
}

/// Captures the text currently selected in the foreground app.
pub fn capture() -> Result<String, SelectionError> {
    Err(SelectionError::NotImplemented)
}

/// Replaces the foreground selection with `text`.
#[allow(dead_code)]
pub fn replace(_text: &str) -> Result<(), SelectionError> {
    Err(SelectionError::NotImplemented)
}
