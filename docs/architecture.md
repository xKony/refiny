# Architecture

Status: pre-alpha. The core pipeline and Tauri shell are scaffolded; selection capture is stubbed.

## Process model

A single user-level Tauri application. The Rust backend owns the hotkey, configuration, provider calls, and window management. Two webview windows share one static frontend:

- **main** — settings, status, and a manual "try the pipeline" box.
- **overlay** — transparent, always-on-top mascot shown during a polish action.

No admin rights, no system service. Tray integration and startup-at-login are planned.

## Data flow

1. **Hotkey pressed.** The Tauri global-shortcut handler checks that no action is in flight.
2. **Capture.** Snapshot the clipboard, clear it, send the platform copy shortcut, poll for text (up to 300 ms), restore the snapshot.
3. **Prompt.** Build the system prompt from the user's template plus the preservation list.
4. **Provider.** `POST {base_url}/chat/completions` with a system message and the selected text as the user message. Temperature 0. The whole action is bounded by `timeout_ms`.
5. **Replace.** Put the result on the clipboard, send the platform paste shortcut, wait for the paste, restore the previous clipboard contents.
6. **Overlay.** `idle → working → success | error → idle`.

Today, step 2 returns `SelectionError::NotImplemented`; the hotkey still exercises the overlay and event pipeline, and the settings window can call the provider directly.

## Workspace layout

| Path | Responsibility | Status |
| --- | --- | --- |
| `crates/refiny-core` | Config, prompt builder, provider client. No UI or OS code. | scaffolded |
| `src-tauri/src/lib.rs` | Tauri builder, plugin registration, hotkey setup. | scaffolded |
| `src-tauri/src/commands.rs` | Commands and the polish action pipeline. | scaffolded |
| `src-tauri/src/state.rs` | Config path resolution and shared state. | scaffolded |
| `src-tauri/src/selection.rs` | Platform boundary for capture and replace. | stub |
| `src-tauri/src/tray.rs` | Tray icon and menu. | planned |
| `src/` | Static frontend: settings and overlay. | scaffolded |

## Platform implementation notes

### Windows

- **Hotkey.** Tauri's global-shortcut plugin uses `RegisterHotKey` under the hood on a dedicated message loop.
- **Synthetic input.** Use `SendInput` with explicit key-down/key-up events. `enigo` is the planned wrapper.
- **Clipboard.** `OpenClipboard` can fail transiently; retry with backoff. Preserve at least `CF_UNICODETEXT` and restore it afterwards. `arboard` is the planned wrapper.
- **Elevation.** User Interface Privilege Isolation blocks synthetic input into elevated windows. Detect the foreground window's integrity level and fail with a clear message instead of retrying.
- **Timing.** Allow ~50 ms between modifier transitions; poll the clipboard every 10 ms up to 300 ms.

### macOS

- Synthetic input requires the user to grant Accessibility permission (System Settings → Privacy & Security → Accessibility).
- The app must be signed and notarized for distribution outside the App Store.
- Use `CGEvent` for copy/paste; `enigo` wraps this.
- Transparent windows require `macOSPrivateApi` in the Tauri config.

### Linux

- X11: global hotkeys and synthetic input work through XTest; `enigo` and the global-shortcut plugin support this.
- Wayland: compositors generally block global hotkeys and synthetic input. Treat Linux as best-effort and document the limitation. A future fallback can be a manual "copy, polish, paste" flow.
- Packaging needs WebKitGTK, `libappindicator`, and `librsvg2`.

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
- The overlay is shown by the backend before the action starts and hidden by the frontend after a terminal state.

## Testing

- **Unit:** prompt builder, config round-trip and validation, provider client against `httpmock`.
- **Integration:** fake provider and fake selection service behind the `Polisher` and selection boundaries.
- **Manual matrix:** Notepad, VS Code, Chrome, Word, Slack, Windows Terminal; 100% and 150% DPI; multi-monitor; macOS Accessibility off/on; X11 and Wayland sessions.

## Open decisions

- Clipboard round-trip vs. accessibility-API capture (UI Automation, AXUIElement, AT-SPI).
- Wayland fallback behavior.
- Tray menu shape and first-run onboarding.
- Auto-update mechanism and code signing.
