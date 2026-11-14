# Architecture

Status: pre-alpha. The hotkey path has been run end to end on Windows only; macOS and Linux compile but have never been launched. See [Platform status](../README.md#platform-status) for the full statement. There are no releases and no published installers.

## Process model

A single user-level Tauri application, enforced by the single-instance plugin: a second launch restores the first window and exits before it touches the hotkey, tray, or config. The Rust backend owns the hotkey, the live configuration, the secret store, provider calls, and window management.

Two webview windows are declared in `src-tauri/tauri.conf.json` with `create: false` and built during setup:

- **main** - settings in four tabs (Provider, Hotkey, Prompts, Playground) plus a status bar.
- **overlay** - transparent, always-on-top, undecorated, mascot shown during a polish action.

Both load pages from `src/` and share `styles.css`. Neither is a framework build; the frontend is static HTML, CSS, and JS embedded at compile time.

The window is a fixed-height frame: the masthead, status bar, and tabs never move, and only the active panel's body scrolls. Each panel pins its action bar below that scroll area, so Save is always reachable.

A tray icon keeps the hotkey alive after the settings window is closed. Closing **main** hides it; **Open settings** or a left-click on the icon restores it; **Quit** exits the process. The tray menu also lists the prompt presets and marks the active one. On macOS, `RunEvent::Reopen` also restores **main** when the dock icon is clicked; like the rest of the macOS path, it has never been exercised. No admin rights and no system service.

The setup banner appears until the configuration has been saved once. That is `Config.setup_complete`, not the file's existence: every install creates the file, so its presence proves nothing. Dismissing the banner sets a flag in the webview's `localStorage`.

## Data flow

A polish action is orchestrated by `PolishAction` in `crates/refiny-core/src/pipeline.rs`, behind two seams: `Selection` (capture and replace) and `PolishProvider`. The Tauri shell supplies the real implementations; tests supply fakes.

1. **Hotkey pressed.** `PolishAction::run` takes the busy flag. If an action is already running, it returns `ActionError::Busy` and the caller ignores it.
2. **Capture.** Snapshot the clipboard, wait for the hotkey modifiers to be released (up to 600 ms), clear the clipboard, send the platform copy shortcut, then poll for text every 12 ms up to 800 ms. Restore the snapshot. Selections over 20,000 characters are rejected.
3. **Prompt.** Build the system prompt from the active prompt template plus the preservation list.
4. **Provider.** `POST {base_url}/chat/completions` with a system message and the selected text as the user message. Temperature 0. The request is bounded by `timeout_ms`. The API key is resolved per call: stored key first, then the environment variable.
5. **Replace.** Put the result on the clipboard, wait for modifiers, send the platform paste shortcut, wait 300 ms for the target app to read the clipboard, then restore the snapshot. Empty results are refused. A target app that reads the clipboard later than 300 ms pastes the restored contents instead, and Refiny still reports success.
6. **Overlay.** Shown as soon as an action reports a state. A successful capture emits `working`, then `success` or `error`. The frontend hides the overlay 1800 ms after a terminal state.

Capture runs before the overlay is shown, so the overlay can never take focus away from the app that owns the selection. The overlay window is non-activating (`focus: false`) for the same reason. Every path that takes a clipboard snapshot restores it, including failures.

## Workspace layout

| Path | Responsibility |
| --- | --- |
| `crates/refiny-core/src/config.rs` | TOML shape, validation, save, prompt-template resolution. |
| `crates/refiny-core/src/config_store.rs` | The `ConfigStore` seam, the file and in-memory stores, and `LiveConfig`. |
| `crates/refiny-core/src/secrets.rs` | The `SecretStore` seam, the key resolution order, and an in-memory store. |
| `crates/refiny-core/src/selection.rs` | The `Selection` seam and the selection error taxonomy. |
| `crates/refiny-core/src/pipeline.rs` | `PolishAction`: the orchestration and its reported state sequence. |
| `crates/refiny-core/src/presets.rs` | Provider preset table and the tray prompt-menu data. |
| `crates/refiny-core/src/prompt.rs` | Default system prompt and preservation-list assembly. |
| `crates/refiny-core/src/provider.rs` | `Polisher`, the OpenAI-compatible client, and `ConfiguredProvider`. |
| `src-tauri/src/lib.rs` | Tauri builder, plugins, hotkey registration, close-to-tray. |
| `src-tauri/src/commands.rs` | The command surface the settings window calls. |
| `src-tauri/src/state.rs` | Config path resolution, live state, and wiring the seams. |
| `src-tauri/src/selection.rs` | `OsSelection`: the clipboard round trip and synthetic input. |
| `src-tauri/src/keychain.rs` | `KeychainSecrets`: the OS credential store. |
| `src-tauri/src/tray.rs` | Tray icon, menu, and window restoration. |
| `src/` | Static settings and overlay UI. |

`refiny-core` depends on no Tauri or OS crate, so it builds and tests anywhere Rust runs. Everything platform-specific is an implementation of a core trait.

## Configuration lifecycle

`state::load` resolves Tauri's app config directory, creates `config.toml` with defaults when it is absent, and records `first_run`. The result is a `LiveConfig`: a `ConfigStore` plus the current value behind a lock.

`AppState` holds `Arc<LiveConfig>`, so every command and every polish action reads the value in effect rather than a copy from startup.

- `LiveConfig::update` applies an edit to a copy and saves it. A value `Config::validate` rejects never reaches the file or the running app.
- `Config::save` validates, writes a sibling temporary file, then renames it over the target. An interrupted write cannot leave a truncated config.
- `LiveConfig::reload` re-reads the file. Nothing calls it yet: there is no file watcher, so a hand edit to `config.toml` still needs a restart.

The hotkey is parsed and registered during setup, and `save_hotkey` changes it in place. The API key is resolved on every request, so a key stored in the settings window is used immediately.

## Platform implementation notes

### Windows

- **Hotkey.** Tauri's global-shortcut plugin wraps the `global-hotkey` crate, which calls `RegisterHotKey` against a hidden, no-activate window; the plugin marshals registration onto the app's main thread.
- **Synthetic input.** `enigo` sends `SendInput` events with explicit key-down and key-up transitions.
- **Clipboard.** `arboard` snapshots text, then an image if there is no text. The clipboard is cleared just before the copy is sent, so a silent copy (nothing selected) is detectable, then restored afterwards.
- **Key storage.** `keyring` 4.x uses the Windows Credential Manager, keyed by the service `dev.refiny.app` and the account `api-key`.
- **Elevation.** User Interface Privilege Isolation blocks synthetic input into elevated windows. The failure is reported as "Nothing was selected"; there is no integrity-level probe.
- **Timing.** 600 ms for the modifiers, 12 ms polling up to 800 ms for capture, 300 ms paste settle.

### macOS

Nothing here has been run. What the code does, and what the platform requires:

- Synthetic input requires the user to grant Accessibility permission in System Settings, Privacy & Security, Accessibility. `device_query` panics without it, and the code catches the panic and falls back to a fixed 150 ms delay before sending input.
- The keychain backend is Keychain Services.
- Distribution outside the App Store would require signing and notarization. None of that exists.
- `enigo` maps `Key::Unicode('c')` and `Key::Unicode('v')` to layout-dependent keycodes off Windows.

### Linux

Nothing here has been run.

- X11 is the assumption: the `global-hotkey` crate registers via `XGrabKey` and opens a direct X11 connection, failing with a clear error if it cannot. `enigo` sends synthetic input through XTest.
- Wayland compositors generally block both. A Wayland session is expected to fail at capture; the user message says so.
- The keychain backend is the Secret Service over D-Bus. A session without one reports the keychain as unavailable and falls back to the environment variable, so the app still runs.
- Packaging would need WebKitGTK, `libappindicator` or `libayatana-appindicator`, and `librsvg2`. If the tray cannot be built, setup returns that error and the process exits at startup instead of running invisibly. CI installs `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libgtk-3-dev`, and `libx11-dev` before checking the shell.

## Failure taxonomy

Every failure reaches the overlay as text derived from the error: the `Display` string, except "nothing selected", which is rewritten per platform. Capture and provider failures happen before any text is written, so the selection stays as it was.

| Failure | Message shown | Text state |
| --- | --- | --- |
| Nothing selected, Windows | "Nothing was selected. Elevated windows and some apps block synthetic input." | unchanged |
| Nothing selected, macOS | "Nothing was selected. If this keeps happening, grant Refiny Accessibility permission in System Settings, Privacy & Security, Accessibility." | unchanged |
| Nothing selected, Linux | "Nothing was selected. Wayland sessions may block synthetic input; X11 is required." | unchanged |
| Selection over 20,000 characters | "the selection is longer than 20000 characters" | unchanged |
| Provider returned a non-2xx status | "provider returned HTTP {status}: {first 300 chars}" | unchanged |
| Provider returned an `error` object | "provider returned an error: {message}" | unchanged |
| No choices or empty content | "provider returned no choices" / "provider returned empty text" | unchanged |
| Network failure or timeout | "provider request failed: {reqwest error}" | unchanged |
| Clipboard or synthetic-input failure | "could not access the clipboard: ..." / "could not send synthetic input: ..." | unchanged; clipboard restored when the snapshot was taken |
| Another action already running | nothing shown; the press is ignored | unchanged |

There is no retry. A retry could paste stale text into a different context.

`selection` also rejects empty text ("refusing to paste empty text"), but the provider client rejects an empty response first, so in practice that branch does not fire. A replace failure restores the clipboard, which discards the polished text: it is not left there for the user to paste by hand.

Settings failures are separate from action failures and are shown next to the button that caused them:

| Failure | Message shown |
| --- | --- |
| Provider URL empty, or not `http(s)://` | "Provider URL must start with http:// or https://." |
| Model empty | "Model must not be empty." |
| Hotkey unparseable | "`{value}` is not a valid hotkey: {reason}" |
| Hotkey owned by another app | "`{value}` is unavailable: {reason}" |
| Duplicate prompt preset name | "invalid config: duplicate prompt preset name `{name}`" |
| Keychain unavailable while storing a key | "could not reach the OS keychain: {reason}" |

A settings save either applies in full or reports why and leaves the running app as it was.

## Concurrency

- One polish action at a time, guarded by an `AtomicBool` inside `PolishAction` and released by a guard that runs even if the action panics.
- The Playground tab calls the provider directly and does not take that guard, so a manual call can overlap a hotkey action.
- The polish action runs on its own thread; provider calls from the settings window run on `spawn_blocking`. Both block on a synchronous HTTP client.
- Provider calls cannot be cancelled. There is no cancel keybinding.
- The overlay is shown by the backend and hidden by the frontend 1800 ms after a terminal state.
- Hotkey outcomes also go to a **Last action** row in the settings window. It is kept separate from the Playground's status, so a background action never looks like a failure of the button the user just clicked.

## Testing

- **Unit tests, 48, all in `crates/refiny-core`.** Run them with `cargo test -p refiny-core`.
  - config: TOML round trip, save and reload, rejection of an empty model and duplicate preset names, prompt-template fallback, and a 0.0.1 file loading with defaults.
  - config store: a save is visible to the next read, a rejected save changes nothing, reload picks up a hand edit, a corrupt file is reported, and a saved value survives a restart.
  - pipeline: success, capture failure, oversized selection, provider failure, replace failure, a saved config reaching the next action, and the busy case.
  - secrets: store/read/clear, an unavailable keychain, and the resolution order (keychain, environment, none).
  - presets: the table's invariants, matching a config to a preset, the prompt-menu contents, and menu ids mapping back to choices.
  - provider: the real client against an `httpmock` server (success and a 401).
- **Test scratch space lives under `target/test-tmp`**, not the machine temp directory, so the tests do not depend on `TMPDIR` permissions. See `crates/refiny-core/src/test_support.rs`.
- **`src-tauri` has one test, ignored by default:** a round trip through the real OS credential store, run with `cargo test -p refiny --lib keychain -- --ignored`. It stays ignored so CI needs no credential store.
- **Manual, on Windows:** the hotkey path, the settings flows, the tray, and window behaviour. There is no written matrix of foreground apps, DPI settings, or monitors.

CI runs the same two jobs on Windows, macOS, and Ubuntu: one checks formatting, clippy, and the `refiny-core` tests, the other compiles the Tauri shell. The exact commands are in [CONTRIBUTING.md](../CONTRIBUTING.md#build-and-test). A green run proves the shell compiles; it does not execute the app.

## Decisions

Settled choices are recorded as ADRs in `docs/adr/`: [stack](adr/0001-rust-and-tauri.md), [platform policy](adr/0002-platform-support-policy.md), [provider interface](adr/0003-openai-compatible-provider.md), [clipboard round trip](adr/0004-clipboard-round-trip.md), [tray lifecycle](adr/0005-tray-lifecycle.md), [secrets in the keychain](adr/0006-secrets-in-os-keychain.md), [live configuration](adr/0007-live-configuration.md).

Open questions, including which capture mechanism should replace the clipboard round trip, live in the [whitepaper](whitepaper.md#6-open-questions).
