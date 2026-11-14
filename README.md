# Refiny

Select text, press a hotkey, get it back with the grammar, punctuation, and capitalization fixed, without leaving the app you are writing in.

Refiny is a tray application. It reads the selected text through a clipboard round trip, sends it to an OpenAI-compatible chat endpoint, and pastes the result over the selection. Names, technical terms, and each language span are meant to come back unchanged, and a preservation list you maintain takes priority.

**Status: pre-alpha, version 0.0.1, one author, one contributor.** Nothing has been released: no tags, no installers, no packages, and nobody outside the author has installed it. The hotkey path has been run end to end on Windows only. macOS and Linux compile; nobody has launched either. See [Platform status](#platform-status).

## How it works

1. Select text in any app.
2. Press `Ctrl+Alt+P`.
3. A small overlay shows a corgi mascot while the request runs.
4. The result replaces the selection.

If a step fails, the original text is left untouched and the previous clipboard contents are restored.

The settings window shows the hotkey, the provider, whether the API key is visible to the process, and the outcome of the last hotkey run. It also has a **Try the pipeline** box that sends whatever you type through the same code path. Closing the window leaves Refiny in the tray; **Quit** in the tray menu exits.

## Platform status

| Platform | State |
| --- | --- |
| Windows 10/11 | Runs. Verified by hand by the author. |
| macOS | Compiles. `cargo check` and `cargo clippy` pass in CI. Never launched. |
| Linux | Compiles. `cargo check` and `cargo clippy` pass in CI on Ubuntu. Never launched. |

CI builds the Tauri shell on all three platforms and runs the `refiny-core` tests. A green run means the code compiles. It says nothing about whether the app works on macOS or Linux, because the author owns no machine that runs either. Nobody is scheduled to change that.

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
- `mascot` is read into the settings view but has no effect. The overlay always shows the corgi.
- `provider.base_url` takes any endpoint with the `POST {base_url}/chat/completions` shape. Point it at Ollama or LM Studio to keep text on your machine.

**The file is read once, at startup, and nothing in the app writes it.** `Config::save` exists in `refiny-core` but is never called, and no command writes configuration. The workflow today is: edit `config.toml`, quit from the tray, relaunch. The same applies to `hotkey`, which cannot change without a restart.

`provider.api_key_env` names an environment variable, not a value. The key is read from that variable on every request and is never written to `config.toml`. A running process cannot gain an environment variable, so set it before launching Refiny. On first run the settings window shows the command for your platform; on Windows that is:

```powershell
setx REFINY_API_KEY "sk-..."
```

The **Recheck configuration** button re-reads the running process's environment, so it cannot see a variable set after launch. Restart Refiny after setting one.

## Build

Prerequisites are Rust stable (edition 2024) and, for `cargo tauri dev`, the Tauri 2 CLI. Platform packages are listed in [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
cargo install tauri-cli --version "^2"
cargo test -p refiny-core
cargo tauri dev
```

The frontend is embedded at build time, so `cargo run -p refiny` also launches the app without the Tauri CLI.

## Project layout

```
crates/refiny-core/  Config, prompt construction, provider client. No UI or OS code.
src-tauri/           Tauri shell: hotkey, tray, windows, commands, selection.
src/                 Settings and overlay UI. Plain HTML, CSS, and JS, embedded at build time.
docs/                Whitepaper, architecture, ADRs.
```

[docs/architecture.md](docs/architecture.md) describes what each module owns.

## Roadmap

Work lives in GitHub issues. There is no dated plan, and nothing below is scheduled.

- Done, on Windows: hotkey, selection capture and replace, overlay, tray lifecycle, first-run setup, and the provider client.
- Open: [issue #1](https://github.com/xKony/refiny/issues/1) covers making Refiny configurable inside the app: saving settings from the window, storing the key in the OS keychain, changing the hotkey without a restart, provider presets, and prompt presets.
- Open: [issue #4](https://github.com/xKony/refiny/issues/4) adds the seams needed to test the polish action.
- Not started and not planned: runtime verification on macOS and Linux, Wayland support, streaming, cancel, history, per-app profiles, packaging, installers, code signing, and auto-update.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Issues and pull requests are welcome. Open tasks carry the `ready-for-agent` label.

## License

MIT. See [LICENSE](LICENSE).
