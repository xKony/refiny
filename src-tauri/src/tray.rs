//! Tray icon, tray menu, and the window helpers shared by every entry point.

use refiny_core::presets;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::commands;
use crate::state::AppState;

/// Menu item id for the read-only "Prompt: ..." line.
const ACTIVE_ID: &str = "active";

/// Builds the tray icon and its menu.
///
/// The tray is what keeps the app discoverable after the settings window is
/// closed: left-click or "Open settings" restores it, "Quit" exits the app.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let menu = menu(app)?;

    let mut builder = TrayIconBuilder::with_id("refiny")
        .tooltip(tooltip(app))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id.as_ref();
            if id.starts_with("preset:") {
                select_preset(app, id);
                return;
            }

            match id {
                "open" => show_main(app),
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

/// Rebuilds the menu and tooltip after the presets or the active one changed.
///
/// The menu is attached to the existing icon, so only the menu and the tooltip
/// are replaced.
pub fn refresh(app: &AppHandle) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id("refiny") else {
        return Ok(());
    };
    tray.set_menu(Some(menu(app)?))?;
    tray.set_tooltip(Some(tooltip(app)))?;
    Ok(())
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let config = app.state::<AppState>().config.snapshot();

    let open = MenuItem::with_id(app, "open", "Open settings", true, None::<&str>)?;
    let active = MenuItem::with_id(
        app,
        ACTIVE_ID,
        format!("Prompt: {}", presets::prompt_menu_summary(&config)),
        false,
        None::<&str>,
    )?;
    let separator_top = PredefinedMenuItem::separator(app)?;

    let prompt_items: Vec<CheckMenuItem<Wry>> = presets::prompt_menu(&config)
        .into_iter()
        .map(|item| {
            CheckMenuItem::with_id(app, item.id, item.label, true, item.checked, None::<&str>)
        })
        .collect::<tauri::Result<_>>()?;

    let separator_bottom = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let mut parts: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = vec![&open, &separator_top, &active];
    for item in &prompt_items {
        parts.push(item);
    }
    parts.push(&separator_bottom);
    parts.push(&quit);

    Menu::with_items(app, &parts)
}

fn tooltip(app: &AppHandle) -> String {
    let config = app.state::<AppState>().config.snapshot();
    format!("Refiny - prompt: {}", presets::prompt_menu_summary(&config))
}

/// Switches the active prompt, driven by a tray click.
fn select_preset(app: &AppHandle, menu_id: &str) {
    let config = app.state::<AppState>().config.snapshot();

    // A preset can be deleted between the menu being built and clicked, so an
    // id that no longer resolves is ignored rather than guessed at.
    let Some(choice) = presets::preset_for_menu_id(&config, menu_id) else {
        return;
    };
    let name = choice.unwrap_or_default();

    if let Err(error) = commands::switch_preset(app, &name) {
        eprintln!("could not switch prompt preset: {error}");
    }
}

/// Restores, focuses, and raises the settings window.
pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
