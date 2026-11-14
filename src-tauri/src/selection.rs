//! The real OS selection: a clipboard round trip and synthetic input.
//!
//! This module is the only place that touches the clipboard or sends keystrokes.
//! It implements the core [`Selection`] seam so the polish action can be tested
//! with a fake.

use std::panic;
use std::thread::sleep;
use std::time::{Duration, Instant};

use arboard::{Clipboard, ImageData};
use device_query::{DeviceQuery, DeviceState, Keycode};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use refiny_core::{MAX_CHARS, Selection, SelectionError};

/// How long to wait for the target app to answer a copy.
const CAPTURE_TIMEOUT: Duration = Duration::from_millis(800);
const POLL_INTERVAL: Duration = Duration::from_millis(12);
/// How long to wait for the hotkey modifiers to be physically released.
const MODIFIER_TIMEOUT: Duration = Duration::from_millis(600);
/// How long to let the target app consume a paste before restoring the clipboard.
const PASTE_SETTLE: Duration = Duration::from_millis(300);

/// The clipboard round trip, as the core seam expects it.
pub struct OsSelection;

impl Selection for OsSelection {
    fn capture(&self) -> Result<String, SelectionError> {
        capture()
    }

    fn replace(&self, text: &str) -> Result<(), SelectionError> {
        replace(text)
    }
}

/// Captures the text currently selected in the foreground app.
fn capture() -> Result<String, SelectionError> {
    let mut clipboard = Clipboard::new().map_err(clipboard_error)?;
    let mut snapshot = snapshot(&mut clipboard);

    let result = (|| {
        wait_for_modifiers_released();
        // Clear only once the copy is about to be sent, so the clipboard stays
        // intact while the user is still releasing the hotkey. A silent copy
        // (nothing selected) then leaves the clipboard empty and is detectable.
        clipboard.clear().map_err(clipboard_error)?;
        send_shortcut(COPY_KEY)?;
        poll_text(&mut clipboard)
    })();

    restore(&mut clipboard, &mut snapshot);
    result
}

/// Replaces the foreground selection with `text`.
fn replace(text: &str) -> Result<(), SelectionError> {
    if text.is_empty() {
        return Err(SelectionError::Empty);
    }

    let mut clipboard = Clipboard::new().map_err(clipboard_error)?;
    let mut snapshot = snapshot(&mut clipboard);

    let result = (|| {
        wait_for_modifiers_released();
        clipboard.set_text(text).map_err(clipboard_error)?;
        send_shortcut(PASTE_KEY)?;
        // Give the target app time to read the clipboard before restoring it.
        sleep(PASTE_SETTLE);
        Ok(())
    })();

    restore(&mut clipboard, &mut snapshot);
    result
}

/// What the clipboard held before an action.
struct Snapshot {
    text: Option<String>,
    image: Option<ImageData<'static>>,
}

fn snapshot(clipboard: &mut Clipboard) -> Snapshot {
    match clipboard.get_text() {
        Ok(text) => Snapshot {
            text: Some(text),
            image: None,
        },
        Err(_) => Snapshot {
            text: None,
            image: clipboard.get_image().ok(),
        },
    }
}

fn restore(clipboard: &mut Clipboard, snapshot: &mut Snapshot) {
    // Best effort: the action result matters more than a restore failure.
    if let Some(text) = snapshot.text.take() {
        let _ = clipboard.set_text(text);
    } else if let Some(image) = snapshot.image.take() {
        let _ = clipboard.set_image(image);
    } else {
        let _ = clipboard.clear();
    }
}

fn poll_text(clipboard: &mut Clipboard) -> Result<String, SelectionError> {
    let deadline = Instant::now() + CAPTURE_TIMEOUT;
    while Instant::now() < deadline {
        if let Ok(text) = clipboard.get_text()
            && !text.trim().is_empty()
        {
            let length = text.chars().count();
            if length > MAX_CHARS {
                return Err(SelectionError::TooLong(MAX_CHARS));
            }
            return Ok(text);
        }
        sleep(POLL_INTERVAL);
    }
    Err(SelectionError::NothingSelected)
}

fn send_shortcut(key: Key) -> Result<(), SelectionError> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|error| SelectionError::Input(error.to_string()))?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };

    enigo.key(modifier, Direction::Press).map_err(input_error)?;
    let pressed = enigo.key(key, Direction::Click);
    let _ = enigo.key(modifier, Direction::Release);
    pressed.map_err(input_error)
}

/// `enigo` exposes letter key variants only on Windows. Elsewhere the Unicode
/// form is mapped to a layout-dependent keycode.
#[cfg(target_os = "windows")]
const COPY_KEY: Key = Key::C;
#[cfg(not(target_os = "windows"))]
const COPY_KEY: Key = Key::Unicode('c');

#[cfg(target_os = "windows")]
const PASTE_KEY: Key = Key::V;
#[cfg(not(target_os = "windows"))]
const PASTE_KEY: Key = Key::Unicode('v');

/// Waits until the hotkey modifiers are released, so `Ctrl+C` is not sent as
/// `Ctrl+Alt+C` while the user is still holding the shortcut.
///
/// If key state cannot be queried (Wayland, missing macOS permission), fall
/// back to a short fixed delay.
fn wait_for_modifiers_released() {
    let deadline = Instant::now() + MODIFIER_TIMEOUT;
    loop {
        match modifiers_down() {
            Some(false) => return,
            Some(true) if Instant::now() < deadline => sleep(Duration::from_millis(10)),
            Some(true) => return,
            None => {
                sleep(Duration::from_millis(150));
                return;
            }
        }
    }
}

fn modifiers_down() -> Option<bool> {
    // `DeviceState::new` panics when there is no X display or, on macOS, when
    // Accessibility permission is missing. Treat that as "unknown".
    let state = panic::catch_unwind(DeviceState::new).ok()?;
    Some(state.get_keys().iter().any(is_modifier))
}

fn is_modifier(key: &Keycode) -> bool {
    matches!(
        key,
        Keycode::LControl
            | Keycode::RControl
            | Keycode::LShift
            | Keycode::RShift
            | Keycode::LAlt
            | Keycode::RAlt
            | Keycode::LMeta
            | Keycode::RMeta
            | Keycode::Command
            | Keycode::RCommand
    )
}

fn clipboard_error(error: arboard::Error) -> SelectionError {
    SelectionError::Clipboard(error.to_string())
}

fn input_error(error: enigo::InputError) -> SelectionError {
    SelectionError::Input(error.to_string())
}
