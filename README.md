# Refiny

Select text, press a hotkey, get it back with the grammar, punctuation, and capitalization fixed, without leaving the app you are writing in.

Refiny is a tray application. It reads the selected text through a clipboard round trip, sends it to an OpenAI-compatible chat endpoint, and pastes the result over the selection. The default prompt asks the model to fix expression while leaving names, technical terms, and each language span alone, and a preservation list you maintain is appended to every request.

**Status: pre-alpha, version 0.0.1, one author, one contributor.** There are no git tags, no GitHub releases, and no installers; nobody outside the author has run it. The hotkey path has been run end to end on Windows only. macOS and Linux compile; nobody has launched either. See [Platform status](#platform-status).

## How it works

1. Select text in any app.
2. Press `Ctrl+Alt+P`, the default hotkey.
3. A small pill beside the pointer shows a pixel mascot while the request runs.
4. The result replaces the selection.

If capture or the provider call fails, the text stays as it was and the clipboard is put back afterwards.

The settings window is the way in. It has four tabs: **Provider** (presets for OpenAI, OpenRouter, Ollama, and LM Studio that fill the URL and model, editable fields, and a connection test), **Hotkey** (changes take effect immediately and keep the old shortcut if the new one is taken), **Prompts** (the default prompt, the preservation list, and named presets you can switch from the tray), and **Playground** (sends text through the same pipeline as the hotkey). Closing the window leaves Refiny in the tray; **Quit** in the tray menu exits.

## Platform status

| Platform | State |
| --- | --- |
| Windows 10/11 | Runs. Verified by hand by the author. |
| macOS | Compiles. `cargo check` and `cargo clippy` pass in CI. Never launched. |
| Linux | Compiles. `cargo check` and `cargo clippy` pass in CI on Ubuntu. Never launched. |

CI runs two jobs on Windows, macOS, and Ubuntu. One checks formatting, runs clippy and the `refiny-core` tests. The other compiles the Tauri shell. [CONTRIBUTING.md](CONTRIBUTING.md#build-and-test) lists the commands. A green run means the tests pass and the code builds. It says nothing about whether the app works on macOS or Linux, because the author owns no machine that runs either. Nobody is scheduled to change that.

The interaction depends on a global hotkey, synthetic copy and paste, and a clipboard round trip, all of which are OS-specific. On Linux, X11 is the assumption; Wayland compositors generally block global hotkeys and synthetic input.

## Configuration

Everything is set from the settings window: provider, key, hotkey, prompts, and the preservation list. A save applies to the next polish action without a restart.

The window has four tabs. **Provider** has presets for OpenAI, OpenRouter, Ollama, and LM Studio that fill the URL and model, plus editable fields and a connection test. **Hotkey** changes the shortcut in place and keeps the old one if the new one is taken. **Prompts** edits the default prompt, the preservation list, and named presets. **Playground** sends text through the same pipeline as the hotkey.

Refiny still reads `config.toml` from Tauri's app config directory for the identifier `dev.refiny.app`. On Windows that is `%APPDATA%\dev.refiny.app\config.toml`; the window shows the resolved path. The file is created with defaults on first launch:

```toml
hotkey = "CommandOrControl+Alt+P"
prompt = ""
preserve = []
timeout_ms = 20000
mascot = "corgi"
active_prompt = ""

[provider]
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key_env = "REFINY_API_KEY"
```

- `hotkey` is a Tauri shortcut string. Changing it in the app re-registers it in the running process.
- `prompt` is the default system prompt. Empty means the built-in default in `crates/refiny-core/src/prompt.rs`.
- `preserve` holds terms to keep exactly as written. Blank and duplicate entries are dropped before the list is appended to the prompt.
- `timeout_ms` bounds the provider request.
- `mascot` is accepted and passed to the frontend, but nothing reads it. The pill always shows the same creature.
- `provider.base_url` takes any endpoint with the `POST {base_url}/chat/completions` shape. Point it at Ollama or LM Studio to keep text on your machine.
- `provider.api_key_env` names an environment variable, not a value. It is a fallback for when no key is stored.
- `prompts` holds named prompt templates and `active_prompt` names the one in use. The tray menu switches between them. An empty or unknown name uses `prompt`.

**The file is readable and editable by hand, but nothing watches it.** An edit made outside the app takes effect after a restart; the window is the supported path.

**The API key is not in this file.** It goes to the OS credential store (Windows Credential Manager, macOS Keychain, Linux Secret Service) under the service `dev.refiny.app`. When nothing is stored, Refiny reads the variable named by `provider.api_key_env`, so an existing setup keeps working. A running process cannot gain an environment variable, which is why setting one with `setx` still needs a restart; pasting the key into the window does not.

A local provider needs no key. Leave the key field blank, and the request goes out with no `Authorization` header.

## Build

Prerequisites are Rust stable (edition 2024) and, for `cargo tauri dev`, the Tauri 2 CLI. Platform packages are listed in [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
cargo install tauri-cli --version "^2"
cargo test -p refiny-core
cargo tauri dev
```

The frontend is embedded at build time, so `cargo run -p refiny` also launches the app without the Tauri CLI.

## Project layout

- `crates/refiny-core` - config and its store, the secret store seam, prompt construction, the selection seam, the polish action's orchestration, provider presets, and the provider client. No UI or OS code, so it builds and tests anywhere Rust runs.
- `src-tauri` - the Tauri shell: hotkey, tray, windows, commands, and the two implementations of the OS seams (`selection.rs` for the clipboard round trip, `keychain.rs` for the credential store).
- `src` - settings and overlay UI, plain HTML, CSS, and JS embedded at build time.
- `docs` - the whitepaper, the architecture reference, the ADRs, and agent instructions.
- `DESIGN.md` - the design system: tokens, type, components, and how screenshots are reviewed without burning the agent's context.

[docs/architecture.md](docs/architecture.md#workspace-layout) has the per-file table.

## Roadmap

There is no dated plan and no milestone list. GitHub issues are the only record of intended work.

- Working, on Windows: hotkey, selection capture and replace, overlay, tray, and a complete settings window - provider form with presets and a connection test, OS keychain for the key, hotkey changes without a restart, named prompt presets switchable from the tray, and the playground. The window is drawn in the Marginalia system recorded in [DESIGN.md](DESIGN.md).
- Open: [issue #11](https://github.com/xKony/refiny/issues/11) explores replacing the hand-written provider presets with the models.dev catalog.
- Known failure: [issue #14](https://github.com/xKony/refiny/issues/14) stops the app starting on the author's Windows machine because the WebView2 webview fails to create. The interface work above was checked in a browser against the same HTML, CSS, and JavaScript until that is fixed.
- Not scheduled: runtime verification on macOS or Linux, Wayland support, streaming, cancel, history, per-app profiles, packaging, installers, code signing, and auto-update. None of these has an owner or a date.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). Issues and pull requests are welcome. Open tasks carry the `ready-for-agent` label.

## License

MIT. See [LICENSE](LICENSE).
