# Refiny

Select text, press a hotkey, get it back with the grammar, punctuation, and capitalization fixed, without leaving the app you are writing in.

Refiny is a tray application. It reads the selected text through a clipboard round trip, sends it to an OpenAI-compatible chat endpoint, and pastes the result over the selection. The default prompt asks the model to fix expression while leaving names, technical terms, and each language span alone, and a preservation list you maintain is appended to every request.

**Status: pre-alpha, version 0.0.1, one author, one contributor.** There are no git tags, no GitHub releases, and no installers; nobody outside the author has run it. The hotkey path has been run end to end on Windows only. macOS and Linux compile; nobody has launched either. See [Platform status](#platform-status).

## How it works

1. Select text in any app.
2. Press `Ctrl+Alt+P`.
3. A small overlay shows a corgi mascot while the request runs.
4. The result replaces the selection.

If a step fails, the original text is left untouched and the clipboard is put back afterwards.

The settings window shows the hotkey, the provider, whether the API key is visible to the process, and the outcome of the last hotkey run. It also has a **Try the pipeline** box that sends whatever you type through the same code path. Closing the window leaves Refiny in the tray; **Quit** in the tray menu exits.

## Platform status

| Platform | State |
| --- | --- |
| Windows 10/11 | Runs. Verified by hand by the author. |
| macOS | Compiles. `cargo check` and `cargo clippy` pass in CI. Never launched. |
| Linux | Compiles. `cargo check` and `cargo clippy` pass in CI on Ubuntu. Never launched. |

CI runs the `refiny-core` unit tests on all three platforms and compiles the Tauri shell on each. A green run means the tests pass and the code builds. It says nothing about whether the app works on macOS or Linux, because the author owns no machine that runs either. Nobody is scheduled to change that.

The interaction depends on a global hotkey, synthetic copy and paste, and a clipboard round trip, all of which are OS-specific. On Linux, X11 is the assumption; Wayland compositors generally block global hotkeys and synthetic input.

## Configuration

Refiny reads `config.toml` from Tauri's app config directory for the identifier `dev.refiny.app`. On Windows that is `%APPDATA%\dev.refiny.app\config.toml`; the settings window displays the resolved path. The file is created with defaults on first launch:

```toml
hotkey = "CommandOrControl+Alt+P"
prompt = ""
preserve = []
timeout_ms = 20000
mascot = "corgi"

[provider]
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key_env = "REFINY_API_KEY"
```

- `hotkey` is a Tauri shortcut string. It is registered once at startup.
- `prompt` is the system prompt. Empty means the built-in default in `crates/refiny-core/src/prompt.rs`.
- `preserve` holds terms to keep exactly as written. Blank and duplicate entries are dropped before the list is appended to the prompt.
- `timeout_ms` bounds the provider request.
- `mascot` is accepted and passed to the frontend, but nothing reads it. The overlay always shows the corgi.
- `provider.base_url` takes any endpoint with the `POST {base_url}/chat/completions` shape. Point it at Ollama or LM Studio to keep text on your machine.

**The file is read once, at startup, and nothing in the app ever updates it.** It is written exactly once, on the first run that creates it. `Config::save` exists in `refiny-core` and is called only by `load_or_create`; no command writes configuration, so an edit to an existing file does nothing until you quit from the tray and relaunch. The same applies to `hotkey`, which cannot change without a restart.

`provider.api_key_env` names an environment variable, not a value. The key is read from that variable on every request and is never written to `config.toml`. A running process cannot gain an environment variable, so set it before launching Refiny. Until you dismiss it, the setup panel appears whenever the key is missing and shows the command for your platform; on Windows that is:

```powershell
setx REFINY_API_KEY "sk-..."
```

The **Recheck configuration** button re-reads the process environment at that moment. `setx` writes to the registry, which a running process does not observe, so the panel keeps reporting the key as missing. Restart Refiny after setting one.

## Build

Prerequisites are Rust stable (edition 2024) and, for `cargo tauri dev`, the Tauri 2 CLI. Platform packages are listed in [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
cargo install tauri-cli --version "^2"
cargo test -p refiny-core
cargo tauri dev
```

The frontend is embedded at build time, so `cargo run -p refiny` also launches the app without the Tauri CLI.

## Project layout

- `crates/refiny-core` - config, prompt construction, provider client. No UI or OS code, so it builds and tests anywhere Rust runs.
- `src-tauri` - the Tauri shell: hotkey, tray, windows, commands, and the platform-specific selection code.
- `src` - settings and overlay UI, plain HTML, CSS, and JS embedded at build time.
- `docs` - the whitepaper, the architecture reference, and the ADRs.

[docs/architecture.md](docs/architecture.md#workspace-layout) has the per-file table.

## Roadmap

There is no dated plan and no milestone list. GitHub issues are the only record of intended work.

- Working, on Windows: hotkey, selection capture and replace, overlay, tray lifecycle, first-run setup, and the provider client.
- Open, unscheduled: [issue #1](https://github.com/xKony/refiny/issues/1) makes Refiny configurable from inside the app (saving settings, OS keychain for the key, hotkey changes without a restart, provider and prompt presets). [Issue #4](https://github.com/xKony/refiny/issues/4) adds the seams needed to test the polish action.
- Not planned: runtime verification on macOS or Linux, Wayland support, streaming, cancel, history, per-app profiles, packaging, installers, code signing, and auto-update. None of these has an owner or a date.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Issues and pull requests are welcome. Open tasks carry the `ready-for-agent` label.

## License

MIT. See [LICENSE](LICENSE).
