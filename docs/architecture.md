# Architecture

Status: pre-alpha. The core pipeline, hotkey, overlay, selection capture/replace, tray lifecycle, and first-run setup are implemented. Windows is runtime-verified; macOS and Linux are compile-verified.

## Process model

A single user-level Tauri application, enforced by the single-instance plugin: launching a second copy restores the first window and exits. The Rust backend owns the hotkey, configuration, provider calls, and window management. Two webview windows share one static frontend:

- **main** — settings, status, and a manual "try the pipeline" box.
- **overlay** — transparent, always-on-top mascot shown during a polish action.

A tray icon keeps the hotkey alive after the settings window is closed. Closing **main** hides it; **Open settings** (or a left-click on the icon) restores it; **Quit** exits the process. On macOS, `RunEvent::Reopen` also restores **main** when the dock icon is clicked. No admin rights, no system service. Startup-at-login is planned.

On first run the backend reports `first_run` (the config file did not exist) and the settings UI shows a setup panel: how to set the API-key environment variable, what the hotkey does, and that the app lives in the tray. The panel also appears whenever the configured key is missing and stays dismissable, so a local provider without a key is not nagged.

## Data flow

1. **Hotkey pressed.** The Tauri global-shortcut handler checks that no action is in flight.
2. **Capture.** Snapshot the clipboard, clear it, wait for the hotkey modifiers to be released, send the platform copy shortcut, poll for text (up to 800 ms), restore the snapshot.
3. **Prompt.** Build the system prompt from the user's template plus the preservation list.
4. **Provider.** `POST {base_url}/chat/completions` with a system message and the selected text as the user message. Temperature 0. The whole action is bounded by `timeout_ms`.
5. **Replace.** Put the result on the clipboard, send the platform paste shortcut, wait 300 ms for the target app to consume it, restore the previous clipboard contents.
6. **Overlay.** `idle → working → success | error → idle`.

Capture runs before the overlay is shown, so the overlay can never take focus away from the app that owns the selection. The overlay is non-activating (`focus: false`) for the same reason. Every path restores the clipboard snapshot, including failures.

## Workspace layout

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/refiny-core` | Config, prompt builder, provider client. No UI or OS code. | tested |
| `src-tauri/src/lib.rs` | Tauri builder, plugin registration, hotkey setup, close-to-tray. | implemented |
| `src-tauri/src/commands.rs` | Commands and the polish action pipeline. | implemented |
| `src-tauri/src/state.rs` | Config path resolution and shared state. | implemented |
| `src-tauri/src/selection.rs` | Platform boundary for capture and replace. | implemented |
| `src-tauri/src/tray.rs` | Tray icon, tray menu, and window restoration. | implemented |
| `src/` | Static frontend: settings, setup panel, and overlay. | implemented |

## Platform implementation notes

### Windows

- **Hotkey.** Tauri's global-shortcut plugin uses `RegisterHotKey` under the hood on a dedicated message loop.
- **Synthetic input.** `enigo` sends `SendInput` events with explicit key-down/key-up transitions.
- **Clipboard.** `arboard` handles text and image snapshots. The clipboard is cleared before capture so a silent copy (nothing selected) is detectable, then restored afterwards.
- **Elevation.** User Interface Privilege Isolation blocks synthetic input into elevated windows. Detect the foreground window's integrity level and fail with a clear message instead of retrying.
- **Timing.** Wait up to 600 ms for the hotkey modifiers to be released; poll the clipboard every 12 ms up to 800 ms; let a paste settle for 300 ms before restoring the snapshot.

### macOS

- Synthetic input requires the user to grant Accessibility permission (System Settings → Privacy & Security → Accessibility).
- The app must be signed and notarized for distribution outside the App Store.
- Use `CGEvent` for copy/paste; `enigo` wraps this.
- Transparent windows work out of the box on Tauri 2.12.1+; the old `macOSPrivateApi` flag is a no-op there and requires a crate feature if set.

### Linux

- X11: global hotkeys and synthetic input work through XTest; `enigo` and the global-shortcut plugin support this.
- Wayland: compositors generally block global hotkeys and synthetic input. Treat Linux as best-effort and document the limitation. A future fallback can be a manual "copy, polish, paste" flow.
- Packaging needs WebKitGTK, `libappindicator` (or `libayatana-appindicator`), and `librsvg2`. Without an appindicator package the tray icon fails to build and the app exits at startup.

## Failure taxonomy

| Failure | User sees | Text state |
| --- | --- | --- |
| Nothing selected | "Nothing selected" | unchanged |
| Provider auth error | "Check your API key" | unchanged |
| Timeout | "The model took too long" | unchanged |
| Rate limit | "Provider busy" | unchanged |
| Replace failed | "Couldn't paste" + result left on clipboard | original preserved |

No background retries: a retry could paste stale text into a different context.

## Concurrency

- One polish action at a time; the hotkey is ignored while working.
- Provider calls run on `spawn_blocking` so the UI and hotkey loop stay responsive.
- The action carries a timeout from the config; cancel support is planned.
- The overlay is shown by the backend after capture succeeds (or after a capture error) and hidden by the frontend 1800 ms after a terminal state.
- Hotkey outcomes also go to a **Last action** row in the settings window. It is kept separate from the manual test box's status, so a background action never looks like a failure of the box the user just clicked.

## Testing

- **Unit:** prompt builder, config round-trip and validation, provider client against `httpmock`.
- **Integration:** fake provider and fake selection service behind the `Polisher` and selection boundaries.
- **Manual matrix:** Notepad, VS Code, Chrome, Word, Slack, Windows Terminal; 100% and 150% DPI; multi-monitor; macOS Accessibility off/on; X11 and Wayland sessions.

## Open decisions

- Clipboard round-trip vs. accessibility-API capture (UI Automation, AXUIElement, AT-SPI).
- Wayland fallback behavior.
- First-run onboarding beyond the setup panel, autostart, and a hotkey picker.
- Auto-update mechanism and code signing.
