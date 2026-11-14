//! The command surface the settings window talks to.
//!
//! Every write goes through `AppState`, which owns the live config, the secret
//! store, and the registered hotkey. A command either applies the whole change
//! or reports why and leaves the running app as it was.

use refiny_core::{
    Config, PromptPreset, ProviderPreset, ResolvedKey, SecretError, State, StateEvent, presets,
    prompt, secrets,
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::state::AppState;
use crate::tray;

/// What the settings window renders.
#[derive(Debug, Serialize)]
pub struct ConfigView {
    pub version: String,
    pub hotkey: String,
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
    /// Where the key comes from: `keychain`, `environment`, or `none`.
    pub key_source: String,
    pub api_key_present: bool,
    pub prompt: String,
    pub preserve: Vec<String>,
    pub mascot: String,
    pub active_prompt: String,
    pub prompts: Vec<PromptPreset>,
    pub presets: Vec<ProviderPreset>,
    /// The preset whose URL and model match the config, if any.
    pub matched_preset: Option<String>,
    pub config_path: String,
    pub first_run: bool,
    pub setup_complete: bool,
}

impl ConfigView {
    fn with_key(
        config: &Config,
        path: &std::path::Path,
        first_run: bool,
        key: ResolvedKey,
    ) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            hotkey: config.hotkey.clone(),
            base_url: config.provider.base_url.clone(),
            model: config.provider.model.clone(),
            api_key_env: config.provider.api_key_env.clone(),
            key_source: key.source.as_str().to_string(),
            api_key_present: key.key.is_some(),
            prompt: config.prompt.clone(),
            preserve: config.preserve.clone(),
            mascot: config.mascot.clone(),
            active_prompt: config.active_prompt.clone(),
            prompts: config.prompts.clone(),
            presets: presets::PRESETS.to_vec(),
            matched_preset: presets::matching(&config.provider.base_url, &config.provider.model)
                .map(|preset| preset.id.to_string()),
            config_path: path.display().to_string(),
            first_run,
            setup_complete: config.setup_complete,
        }
    }
}

/// The current view, built from the live state.
fn current_view(app: &tauri::AppHandle) -> ConfigView {
    let state = app.state::<AppState>();
    let config = state.config.snapshot();
    let key = secrets::resolve(&config, state.secrets.as_ref());
    ConfigView::with_key(&config, &state.config_path(), state.first_run, key)
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> ConfigView {
    current_view(&app)
}

/// The provider section, as posted by the settings form.
#[derive(Debug, Deserialize)]
pub struct ProviderForm {
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
    /// A key to store, or `None` to leave the stored key alone.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Remove the stored key instead of writing one.
    #[serde(default)]
    pub clear_api_key: bool,
}

#[tauri::command]
pub fn save_provider(app: tauri::AppHandle, form: ProviderForm) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();

    let base_url = form.base_url.trim().to_string();
    let model = form.model.trim().to_string();
    let api_key_env = form.api_key_env.trim().to_string();

    if base_url.is_empty() {
        return Err("Provider URL must not be empty.".into());
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("Provider URL must start with http:// or https://.".into());
    }
    if model.is_empty() {
        return Err("Model must not be empty.".into());
    }

    // The key is written before the config: a form that saved a valid provider
    // but silently dropped the key would be the worse failure.
    let new_key = form
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty());
    if let Some(key) = new_key {
        state
            .secrets
            .set(key)
            .map_err(|error: SecretError| error.to_string())?;
    } else if form.clear_api_key {
        state
            .secrets
            .clear()
            .map_err(|error: SecretError| error.to_string())?;
    }

    state
        .config
        .update(|next| {
            next.provider.base_url = base_url.clone();
            next.provider.model = model.clone();
            next.provider.api_key_env = api_key_env.clone();
            next.setup_complete = true;
        })
        .map_err(|error| error.to_string())?;

    let _ = tray::refresh(&app);
    Ok(current_view(&app))
}

/// The prompt section, as posted by the settings form.
#[derive(Debug, Deserialize)]
pub struct PromptForm {
    pub prompt: String,
    pub preserve: Vec<String>,
    pub prompts: Vec<PromptPreset>,
    pub active_prompt: String,
}

#[tauri::command]
pub fn save_prompts(app: tauri::AppHandle, form: PromptForm) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();

    state
        .config
        .update(|next| {
            next.prompt = form.prompt.clone();
            next.preserve = form.preserve.clone();
            next.prompts = form.prompts.clone();
            next.active_prompt = form.active_prompt.clone();
            next.setup_complete = true;
        })
        .map_err(|error| error.to_string())?;

    let _ = tray::refresh(&app);
    Ok(current_view(&app))
}

#[derive(Debug, Deserialize)]
pub struct HotkeyForm {
    pub hotkey: String,
}

/// Moves the global hotkey to a new combination in the running process.
///
/// The new shortcut is registered while the old one is still held, so a
/// combination another application owns is rejected and the current hotkey
/// keeps working. The config is written only after registration succeeds, so a
/// restart cannot resurrect a value that never worked.
#[tauri::command]
pub fn save_hotkey(app: tauri::AppHandle, form: HotkeyForm) -> Result<ConfigView, String> {
    let state = app.state::<AppState>();
    let requested = form.hotkey.trim().to_string();

    if requested.is_empty() {
        return Err("Hotkey must not be empty.".into());
    }

    // Parsing happens before anything is unregistered, so a typo cannot leave
    // the app with no hotkey at all.
    let shortcut: Shortcut = requested.parse().map_err(|error| {
        format!("`{requested}` is not a valid hotkey: {error}. Try something like CommandOrControl+Alt+P.")
    })?;

    let current = *state.hotkey.lock().unwrap();
    if shortcut == current {
        return Ok(current_view(&app));
    }

    app.global_shortcut()
        .register(shortcut)
        .map_err(|error| format!("`{requested}` is unavailable: {error}"))?;

    if let Err(error) = state.config.update(|next| next.hotkey = requested.clone()) {
        let _ = app.global_shortcut().unregister(shortcut);
        return Err(error.to_string());
    }

    let _ = app.global_shortcut().unregister(current);
    *state.hotkey.lock().unwrap() = shortcut;

    Ok(current_view(&app))
}

/// What a connection test needs: the fields as typed, not as saved.
#[derive(Debug, Deserialize)]
pub struct TestForm {
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub message: String,
}

/// Sends a tiny piece of text to the provider named by the form.
///
/// The form is tested as typed, so the user learns their key is wrong before
/// they save it. A key that is not being replaced is resolved the same way a
/// real request would resolve it.
#[tauri::command]
pub async fn test_provider(
    app: tauri::AppHandle,
    form: TestForm,
    state: tauri::State<'_, AppState>,
) -> Result<TestResult, String> {
    let base_url = form.base_url.trim().to_string();
    let model = form.model.trim().to_string();

    if base_url.is_empty() || model.is_empty() {
        return Ok(TestResult {
            ok: false,
            message: "Fill in the provider URL and model first.".into(),
        });
    }

    let typed_key = form
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string);

    let secrets = state.secrets.clone();
    let config = state.config.snapshot();
    let _ = &app;

    let outcome = tauri::async_runtime::spawn_blocking(move || {
        use refiny_core::{OpenAiCompatible, Polisher};

        let key = match typed_key {
            Some(key) => Some(key),
            None => secrets::resolve(&config, secrets.as_ref()).key,
        };

        let client = OpenAiCompatible::from_config(&config, key)?;
        client.polish(
            &prompt::build_prompt("You are a connection test. Reply with exactly: ok", &[]),
            "ok",
        )
    })
    .await
    .map_err(|error| error.to_string())?;

    Ok(match outcome {
        Ok(_) => TestResult {
            ok: true,
            message: "The provider answered.".into(),
        },
        Err(error) => TestResult {
            ok: false,
            message: error.to_string(),
        },
    })
}

/// Sends typed text through the same pipeline the hotkey uses.
#[tauri::command]
pub async fn polish_text(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.snapshot();
    let secrets = state.secrets.clone();

    tauri::async_runtime::spawn_blocking(move || {
        use refiny_core::PolishProvider;

        let provider = refiny_core::ConfiguredProvider::new(secrets);
        let system_prompt = prompt::build_prompt(config.prompt_template(), &config.preserve);
        provider
            .polish(&config, &system_prompt, &text)
            .map_err(|error| error.to_string())
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
    let action = app.state::<AppState>().action.clone();
    let app = app.clone();

    std::thread::spawn(move || {
        let mut report = |event: StateEvent| {
            if matches!(event.state, State::Working | State::Success | State::Error) {
                show_overlay(&app);
            }
            let _ = app.emit("refiny-state", event);
        };

        // A press while polishing is ignored rather than reported: the user is
        // holding a hotkey, not asking for an error message.
        let _ = action.run(&mut report);
    });
}

/// Switches the active prompt preset and records the choice.
pub fn switch_preset(app: &tauri::AppHandle, name: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    state
        .config
        .update(|next| next.active_prompt = name.to_string())
        .map_err(|error| error.to_string())?;
    let _ = app.emit("refiny-config-changed", ());
    let _ = tray::refresh(app);
    Ok(())
}

fn show_overlay(app: &tauri::AppHandle) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.show();
    }
}
