mod commands;
mod keychain;
mod selection;
mod state;
mod tray;

use tauri::Manager;
#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use tray::show_main;

pub fn run() {
    tauri::Builder::default()
        // Register the single-instance plugin first so a second launch exits
        // before it touches the hotkey, the tray, or the config.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
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
            commands::save_provider,
            commands::save_prompts,
            commands::save_hotkey,
            commands::test_provider,
            commands::polish_text,
            commands::hide_overlay,
        ])
        .on_window_event(|window, event| {
            // Closing settings keeps the app alive in the tray so the global
            // hotkey keeps working. Quit goes through the tray menu.
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let (config, first_run) = state::load(&handle)?;

            let hotkey_text = config.snapshot().hotkey;
            let shortcut: Shortcut = hotkey_text.parse().map_err(|error| {
                Box::<dyn std::error::Error>::from(format!(
                    "invalid hotkey `{hotkey_text}` in config: {error}"
                ))
            })?;

            // Fails loudly rather than starting a tray app that answers to
            // nothing: the hotkey is the only way in.
            app.global_shortcut().register(shortcut)?;

            let secrets = state::secrets();
            let action = state::build_action(config.clone(), secrets.clone());

            // Manage the state before any window exists. Tauri normally builds
            // config windows before the setup closure runs, which lets the
            // frontend call commands before `AppState` is managed.
            app.manage(state::AppState::new(
                config, secrets, action, shortcut, first_run,
            ));

            for window_config in app.config().app.windows.clone() {
                tauri::WebviewWindowBuilder::from_config(app.handle(), &window_config)?.build()?;
            }

            tray::build(app.handle())?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Refiny")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                show_main(app);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
