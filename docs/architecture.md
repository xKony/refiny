# Architecture

Status: pre-alpha, version 0.0.1. The full hotkey path (capture, provider call, replace, overlay) has been run end to end on Windows, by hand, by the author. macOS and Linux compile and pass `cargo check` and `cargo clippy` in CI; nobody has launched the app on either platform. There are no releases and no published installers.

## Process model

A single user-level Tauri application, enforced by the single-instance plugin: a second launch restores the first window and exits before it touches the hotkey, tray, or config. The Rust backend owns the hotkey, the configuration value loaded at startup, provider calls, and window management.

Two webview windows are declared in `src-tauri/tauri.conf.json` with `create: false` and built during setup:

- **main** - settings, status, and the "Try the pipeline" box.
- **overlay** - transparent, always-on-top, undecorated, mascot shown during a polish action.

Both load pages from `src/` and share `styles.css`. Neither is a framework build; the frontend is static HTML, CSS, and JS embedded at compile time.

A tray icon keeps the hotkey alive after the settings window is closed. Closing **main** hides it; **Open settings** or a left-click on the icon restores it; **Quit** exits the process. On macOS, `RunEvent::Reopen` also restores **main** when the dock icon is clicked; like the rest of the macOS path, it has never been exercised. No admin rights and no system service.

The settings panel appears when no API key is visible to the process, and its title is "Welcome to Refiny" on the launch that created the config file and "Finish setup" on later launches. Dismissing it sets a flag in the webview's `localStorage`. A local provider that needs no key is therefore not nagged after the first dismissal.

## Data flow

1. **Hotkey pressed.** The global-shortcut handler swaps a busy flag. If an action is already running, the press is ignored.
2. **Capture.** Snapshot the clipboard, wait for the hotkey modifiers to be released (up to 600 ms), clear the clipboard, send the platform copy shortcut, then poll for text every 12 ms up to 800 ms. Restore the snapshot. Selections over 20,000 characters are rejected.
3. **Prompt.** Build the system prompt from the user's template plus the preservation list.
4. **Provider.** `POST {base_url}/chat/completions` with a system message and the selected text as the user message. Temperature 0. The request is bounded by `timeout_ms`.
5. **Replace.** Put the result on the clipboard, wait for modifiers, send the platform paste shortcut, wait 300 ms for the target app to read the clipboard, then restore the snapshot. Empty results are refused.
6. **Overlay.** Shown as soon as capture returns, whether it succeeded or failed. It emits `working` on a successful capture, then `success` or `error`. The frontend hides it 1800 ms after a terminal state.

Capture runs before the overlay is shown, so the overlay can never take focus away from the app that owns the selection. The overlay window is non-activating (`focus: false`) for the same reason. Every path restores the clipboard snapshot, including failures.

## Workspace layout

| Path | Responsibility |
| --- | --- |
| `crates/refiny-core/src/config.rs` | TOML load, create, save (unused), validate, API-key lookup. |
| `crates/refiny-core/src/prompt.rs` | Default system prompt and preservation-list assembly. |
| `crates/refiny-core/src/provider.rs` | `Polisher` trait and the OpenAI-compatible HTTP client. |
| `src-tauri/src/lib.rs` | Tauri builder, plugins, hotkey registration, close-to-tray. |
| `src-tauri/src/commands.rs` | Commands, the polish action, and `refiny-state` events. |
| `src-tauri/src/state.rs` | Config path resolution and shared state. |
| `src-tauri/src/selection.rs` | Platform boundary for capture and replace. |
| `src-tauri/src/tray.rs` | Tray icon, menu, and window restoration. |
| `src/` | Static settings and overlay UI. |

`refiny-core` depends on no Tauri or OS crate, so it builds and tests anywhere Rust runs.

## Configuration lifecycle

`state::load_config` resolves Tauri's app config directory, creates `config.toml` with defaults when it is absent, and records `first_run`. `AppState` stores one `Config` value for the life of the process. Nothing reloads it.

`Config::save` exists and is exercised by a unit test, but no command calls it. There is no command surface for writing configuration, no file watcher, and no reload path. The hotkey is parsed and registered once during setup; changing it requires an edit to `config.toml` and a restart. The API key is read from the environment on each request, which is why setting it also requires a restart.

## Platform implementation notes

### Windows

- **Hotkey.** Tauri's global-shortcut plugin uses `RegisterHotKey` on a dedicated message loop.
- **Synthetic input.** `enigo` sends `SendInput` events with explicit key-down and key-up transitions.
- **Clipboard.** `arboard` snapshots text, then an image if there is no text. The clipboard is cleared just before the copy is sent, so a silent copy (nothing selected) is detectable, then restored afterwards.
- **Elevation.** User Interface Privilege Isolation blocks synthetic input into elevated windows. The failure is reported as "Nothing was selected"; there is no integrity-level probe.
- **Timing.** 600 ms for the modifiers, 12 ms polling up to 800 ms for capture, 300 ms paste settle.

### macOS

Nothing here has been run. What the code does, and what the platform requires:

- Synthetic input requires the user to grant Accessibility permission in System Settings, Privacy & Security, Accessibility. `device_query` panics without it, and the code catches the panic and falls back to a fixed 150 ms delay before sending input.
- Distribution outside the App Store would require signing and notarization. None of that exists.
- `enigo` maps `Key::Unicode('c')` and `Key::Unicode('v')` to layout-dependent keycodes off Windows.

### Linux

Nothing here has been run.

- X11 is the assumption: global hotkeys and synthetic input go through XTest, which `enigo` and the global-shortcut plugin support.
- Wayland compositors generally block both. A Wayland session is expected to fail at capture; the user message says so.
- Packaging would need WebKitGTK, `libappindicator` or `libayatana-appindicator`, and `librsvg2`. Without an appindicator package the tray fails to build and the app exits at startup. CI installs `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libgtk-3-dev`, and `libx11-dev` before checking the shell.

## Failure taxonomy

Every failure reaches the overlay as text derived from the error: the `Display` string, except "nothing selected", which is rewritten per platform. The selection is untouched unless the failure happens after the paste has already landed.

| Failure | Message shown | Text state |
| --- | --- | --- |
| Nothing selected, Windows | "Nothing was selected. Elevated windows and some apps block synthetic input." | unchanged |
| Nothing selected, macOS | Same, plus a pointer to Accessibility permission | unchanged |
| Nothing selected, Linux | Same, plus a note that Wayland may block synthetic input | unchanged |
| Selection over 20,000 characters | "the selection is longer than 20000 characters" | unchanged |
| Provider returned a non-2xx status | "provider returned HTTP {status}: {first 300 chars}" | unchanged |
| Provider returned an `error` object | "provider returned an error: {message}" | unchanged |
| No choices or empty content | "provider returned no choices" / "provider returned empty text" | unchanged |
| Network failure or timeout | "provider request failed: {reqwest error}" | unchanged |
| Clipboard or synthetic-input failure | "could not access the clipboard: ..." / "could not send synthetic input: ..." | unchanged; clipboard restored |

There is no retry. A retry could paste stale text into a different context.

`selection::replace` also rejects empty text ("refusing to paste empty text"), but the provider client rejects an empty response first, so in practice that branch does not fire. A replace failure restores the clipboard, overwriting the polished text; it is not left there for the user to paste by hand.

## Concurrency

- One polish action at a time, guarded by an `AtomicBool` and released by a guard that runs even if the action panics.
- The polish action runs on its own thread; manual provider calls from the settings window run on `spawn_blocking`. Both block on a synchronous HTTP client.
- Provider calls cannot be cancelled. There is no cancel keybinding.
- The overlay is shown by the backend and hidden by the frontend 1800 ms after a terminal state.
- Hotkey outcomes also go to a **Last action** row in the settings window. It is kept separate from the manual test box's status, so a background action never looks like a failure of the box the user just clicked.

## Testing

- **Unit, 7 tests, all in `crates/refiny-core`:** config TOML round trip, `load_or_create` writing defaults, rejection of an empty model, empty template falling back to the default prompt, preservation-list deduplication, and two provider tests against an `httpmock` server (success and a 401).
- **Nothing else is automated.** There is no `tests/` directory, no integration test, and no test in `src-tauri`. `run_pipeline` constructs `OpenAiCompatible` directly and `selection` is free functions, so there is no injection point for a fake provider or a fake selection service. Adding one is [issue #4](https://github.com/xKony/refiny/issues/4).
- **Manual:** the author has run the hotkey path on Windows. There is no written matrix of foreground apps, DPI settings, or monitors.

CI runs `cargo fmt --all -- --check` and `cargo test -p refiny-core` on Windows, macOS, and Ubuntu, and `cargo check -p refiny` plus `cargo clippy -p refiny --all-targets -- -D warnings` on the same three. A green run proves the shell compiles; it does not execute the app.

## Open decisions

- Clipboard round trip versus accessibility APIs (UI Automation, AXUIElement, AT-SPI) for capture.
- Whether any Linux support beyond X11 is worth attempting.
- Auto-update and code signing, if the project ever ships builds.

Settled elsewhere: tray lifecycle in [ADR-0005](adr/0005-tray-lifecycle.md), clipboard round trip in [ADR-0004](adr/0004-clipboard-round-trip.md), provider interface in [ADR-0003](adr/0003-openai-compatible-provider.md).
