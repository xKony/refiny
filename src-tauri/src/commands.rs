use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use refiny_core::prompt::build_prompt;
use refiny_core::{Config, OpenAiCompatible, Polisher};
use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::selection;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct StateEvent {
    pub state: String,
    pub message: String,
}

impl StateEvent {
    fn working() -> Self {
        Self {
            state: "working".into(),
            message: "Polishing…".into(),
        }
    }

    fn success(message: impl Into<String>) -> Self {
        Self {
            state: "success".into(),
            message: message.into(),
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            state: "error".into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConfigView {
    pub version: String,
    pub hotkey: String,
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
    pub api_key_present: bool,
    pub prompt: String,
    pub preserve: Vec<String>,
    pub mascot: String,
    pub config_path: String,
    pub first_run: bool,
}

impl ConfigView {
    fn from_config(config: &Config, config_path: &std::path::Path, first_run: bool) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            hotkey: config.hotkey.clone(),
            base_url: config.provider.base_url.clone(),
            model: config.provider.model.clone(),
            api_key_env: config.provider.api_key_env.clone(),
            api_key_present: config.api_key().is_some(),
            prompt: config.prompt.clone(),
            preserve: config.preserve.clone(),
            mascot: config.mascot.clone(),
            config_path: config_path.display().to_string(),
            first_run,
        }
    }
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> ConfigView {
    ConfigView::from_config(&state.config, &state.config_path, state.first_run)
}

#[tauri::command]
pub async fn polish_text(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.clone();

    tauri::async_runtime::spawn_blocking(move || {
        run_pipeline(&config, &text).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn hide_overlay(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

/// Starts one polish action, ignoring the hotkey while another is running.
pub fn handle_hotkey(app: &tauri::AppHandle) {
    let busy = {
        let state = app.state::<AppState>();
        if state.busy.swap(true, Ordering::SeqCst) {
            return;
        }
        Arc::clone(&state.busy)
    };

    let app = app.clone();
    std::thread::spawn(move || {
        let _guard = BusyGuard(busy);
        run_action(&app);
    });
}

/// Clears the busy flag even if the action panics.
struct BusyGuard(Arc<AtomicBool>);

impl Drop for BusyGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn run_action(app: &tauri::AppHandle) {
    // Capture before showing the overlay so the overlay can never take focus
    // away from the app that owns the selection.
    let text = match selection::capture() {
        Ok(text) => text,
        Err(error) => {
            show_overlay(app);
            emit_state(app, StateEvent::error(error.user_message()));
            return;
        }
    };

    show_overlay(app);
    emit_state(app, StateEvent::working());

    let config = app.state::<AppState>().config.clone();
    let polished = match run_pipeline(&config, &text) {
        Ok(polished) => polished,
        Err(error) => {
            emit_state(app, StateEvent::error(error));
            return;
        }
    };

    match selection::replace(&polished) {
        Ok(()) => emit_state(app, StateEvent::success("Done.")),
        Err(error) => emit_state(app, StateEvent::error(error.user_message())),
    }
}

fn show_overlay(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.show();
    }
}

fn run_pipeline(config: &Config, text: &str) -> Result<String, String> {
    let prompt = build_prompt(&config.prompt, &config.preserve);
    let polisher = OpenAiCompatible::from_config(config).map_err(|error| error.to_string())?;
    polisher
        .polish(&prompt, text)
        .map_err(|error| error.to_string())
}

fn emit_state(app: &tauri::AppHandle, event: StateEvent) {
    let _ = app.emit("refiny-state", event);
}
