# ADR-0005: Tray-resident process lifecycle

- Status: Accepted
- Date: 2026-11-12
- Deciders: xKony

## Context

Refiny's value depends on the global hotkey being registered whenever the user wants to polish text. That makes the process's lifetime a product decision, not a window-management detail. Three shapes were available:

1. **Quit when the settings window closes.** Simplest, and what a default Tauri app does. But it silently disables the hotkey, so a user who tidies their desktop loses the feature they installed the app for.
2. **Stay running with no visible affordance.** The hotkey keeps working, but there is no way to reopen settings or quit without a task manager.
3. **Stay running behind a tray icon.** The process outlives its window, and the tray provides the reopen and quit affordances.

A second question is what a second launch should do. Two instances would fight over the same global hotkey registration and the same config file, and would race on the clipboard during a polish action.

## Decision

Adopt (3): a tray-resident process.

- Closing **main** hides it; it does not exit. `Quit` in the tray menu is the way out.
- The tray menu carries `Open settings` and `Quit`; a left-click on the icon restores settings.
- `tauri-plugin-single-instance` is registered first. A second launch restores and focuses the first window, then exits before it touches the hotkey, the tray, or the config.
- On macOS, `RunEvent::Reopen` restores **main** so clicking the dock icon behaves like clicking the tray icon.
- The tray implementation lives in `src-tauri/src/tray.rs`; the builder wires it up and owns the close-to-tray policy.

## Consequences

- The hotkey survives window closing, which is the behavior the product promises.
- Users must learn that closing the window does not quit. The first-run setup panel states it, and `Quit` is explicit in the tray menu.
- A crashed or force-killed instance leaves no tray icon behind, so the affordance cannot outlive the process.
- Linux needs an appindicator package at build time and a status-area implementation at runtime; without one, the tray fails to build and the app exits at startup rather than running invisibly.
- Startup-at-login is a separate decision and is not implied by any of this.
