# Refiny

**Select text. Hit a hotkey. Get polished writing.**

[![CI](https://github.com/xKony/refiny/actions/workflows/ci.yml/badge.svg)](https://github.com/xKony/refiny/actions/workflows/ci.yml)

Refiny is an open-source desktop companion for people whose thoughts move faster than their keyboard. It fixes grammar, punctuation, and capitalization in place — without switching apps — while keeping names, technical terms, and mixed languages intact.

[Read the whitepaper](docs/whitepaper.md) · [Architecture](docs/architecture.md) · [Contributing](CONTRIBUTING.md)

> **Status: pre-alpha.** The hotkey, overlay, provider pipeline, and selection capture/replacement work end to end on Windows. macOS and Linux are compile-verified; runtime verification is the next milestone.

## Why

Writing in flow means not stopping to fix commas. Most tools ask you to leave the app, paste into a chat window, and carry the result back. Refiny removes that round trip: the text is corrected where it was written.

It is built for messy first drafts — lowercase fragments, missing punctuation, mixed languages, and names that must not be touched.

## How it works

1. Select text anywhere.
2. Press `Ctrl+Alt+P` (`Cmd+Alt+P` on macOS).
3. A low-poly mascot appears while Refiny polishes.
4. The selection is replaced in place.

If anything fails, the original text is left untouched and the previous clipboard contents are restored.

## What it preserves

- Names, brands, and product names
- Code identifiers, URLs, emails, numbers, and units
- Each language span in mixed-language text
- Meaning, tone, and structure

Your own preservation list overrides everything.

## Platform support

| Platform | Status |
| --- | --- |
| Windows 10/11 | First-class target; verified end to end |
| macOS | First-class target (Accessibility permission required); compile-verified |
| Linux | Best-effort: X11 supported, Wayland limited by compositor policy; compile-verified |

## Configuration

Config lives in the platform config directory and is created on first run:

- **Windows:** `%APPDATA%\dev.refiny.app\config.toml`
- **macOS:** `~/Library/Application Support/dev.refiny.app/config.toml`
- **Linux:** `~/.config/dev.refiny.app/config.toml`

```toml
hotkey = "CommandOrControl+Alt+P"
prompt = ""
preserve = ["Refiny", "Kubernetes", "xKony"]
timeout_ms = 20000
mascot = "corgi"

[provider]
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key_env = "REFINY_API_KEY"
```

The prompt is the system prompt. Leave it empty to use the built-in default. The preservation list is appended automatically. Secrets are read from the environment variable named by `provider.api_key_env` — never from the config file.

Any OpenAI-compatible endpoint works. Point `provider.base_url` at Ollama or LM Studio to keep text local.

## Build

Prerequisites: Rust stable (edition 2024), the Tauri 2 CLI, and the platform dependencies listed in [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
cargo install tauri-cli --version "^2"
cargo test -p refiny-core
cargo tauri dev
```

The frontend is embedded at build time, so `cargo run -p refiny` also launches the app without the Tauri CLI — useful for a quick smoke test.

The settings window includes a **Try the pipeline** box that calls the configured provider directly. The hotkey path captures the foreground selection through the clipboard and restores the previous clipboard contents afterwards, so copying and pasting still behave as expected.

## Project layout

```
crates/refiny-core/   Pure logic: config, prompt, provider (no UI, no OS APIs)
src-tauri/            Tauri shell: hotkey, window management, commands
src/                  Overlay and settings UI (plain HTML/CSS/JS)
docs/                 Whitepaper, architecture, ADRs
```

## Roadmap

- [ ] **v0.1 — Windows MVP:** hotkey, overlay, and selection capture/replace done; tray and onboarding pending
- [ ] **v0.2 — Cross-platform:** macOS and Linux (X11), permissions, packaging
- [ ] **v0.3 — Mascot and control:** animated mascot set, cancel, history, prompt presets
- [ ] **v1.0 — Distribution:** installers, signing, auto-update

## Contributing

Issues and PRs are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md); good first tasks carry the `ready-for-agent` label.

## License

MIT © 2026 xKony
