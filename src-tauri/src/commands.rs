use std::time::Duration;

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
}

impl ConfigView {
    fn from_config(config: &Config, config_path: &std::path::Path) -> Self {
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
        }
    }
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> ConfigView {
    ConfigView::from_config(&state.config, &state.config_path)
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

/// Starts one polish action. Selection capture is still a stub, so this
/// currently demonstrates the overlay and event pipeline.
pub fn handle_hotkey(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.show();
    }

    emit_state(app, StateEvent::working());

    let app = app.clone();
    std::thread::spawn(move || match selection::capture() {
        Ok(text) => {
            let config = app.state::<AppState>().config.clone();
            match run_pipeline(&config, &text) {
                Ok(polished) => match selection::replace(&polished) {
                    Ok(()) => emit_state(&app, StateEvent::success("Done.")),
                    Err(error) => emit_state(&app, StateEvent::error(error.to_string())),
                },
                Err(error) => emit_state(&app, StateEvent::error(error)),
            }
        }
        Err(error) => {
            std::thread::sleep(Duration::from_millis(700));
            emit_state(&app, StateEvent::error(error.to_string()));
        }
    });
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
