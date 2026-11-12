mod commands;
mod selection;
mod state;

use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        commands::handle_hotkey(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::polish_text,
            commands::hide_overlay,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let (config, config_path) = state::load_config(&handle)?;

            let shortcut: Shortcut = config.hotkey.parse().map_err(|error| {
                Box::<dyn std::error::Error>::from(format!(
                    "invalid hotkey `{}` in config: {error}",
                    config.hotkey
                ))
            })?;

            app.global_shortcut().register(shortcut)?;

            // Manage the state before any window exists. Tauri normally builds
            // config windows before the setup closure runs, which lets the
            // frontend call commands before `AppState` is managed.
            app.manage(state::AppState::new(config, config_path));

            for window_config in app.config().app.windows.clone() {
                tauri::WebviewWindowBuilder::from_config(app.handle(), &window_config)?.build()?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Refiny");
}
