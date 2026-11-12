# Contributing to Refiny

Thanks for helping make Refiny better. The project is pre-alpha, so the best contributions right now are small, focused, and well explained.

## Prerequisites

- **Rust stable** (edition 2024; `rust-toolchain.toml` pins the stable channel).
- **Tauri 2 CLI:** `cargo install tauri-cli --version "^2"`.
- **Platform dependencies:**
  - **Windows:** WebView2 (preinstalled on Windows 11) and the MSVC build tools.
  - **macOS:** Xcode Command Line Tools.
  - **Linux:** `webkit2gtk-4.1`, `libappindicator3`, `librsvg2`, and `patchelf`. See the Tauri prerequisites guide for your distribution.
- **`gh`** for issue work (optional).

## Build and test

```bash
cargo test -p refiny-core
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo tauri dev
```

`refiny-core` has no UI or OS dependencies, so its tests run on any platform and are the fastest feedback loop.

## Workspace layout

- `crates/refiny-core` — configuration, prompt construction, provider client. Keep it free of Tauri, window, clipboard, and input code.
- `src-tauri` — the Tauri shell. Platform-specific behavior belongs behind a small module boundary, currently `src-tauri/src/selection.rs`.
- `src` — the static frontend (HTML/CSS/JS). No bundler or Node toolchain is required.

Before packaging, regenerate the platform icon set from the source PNG:

```bash
cargo tauri icon src-tauri/icons/icon.png
```

## Configuration during development

The app creates `config.toml` on first run in the platform config directory (see the README). For local experiments, point `provider.base_url` at a local OpenAI-compatible server such as Ollama or LM Studio, and set the API key environment variable named in the config.

Never commit API keys, tokens, or personal text samples.

## Code style

- Run `cargo fmt --all` before committing.
- Treat Clippy warnings as errors in CI.
- Prefer small, typed errors over stringly-typed failures in `refiny-core`.
- Keep the transformation contract in `crates/refiny-core/src/prompt.rs` in sync with the whitepaper.
- Add a test for every behavior change in the prompt builder, config loader, or provider client.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(core): add per-app prompt profiles
fix(selection): restore clipboard after a failed paste
docs: clarify Wayland limitations
```

## Issues and pull requests

- Issues are tracked on GitHub, using the bug and feature templates. The triage labels are `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`.
- Open an issue before a large change so the approach can be discussed.
- Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
- Keep pull requests focused on one logical change, with tests and documentation.
- Describe the user-visible behavior and the failure modes you considered.

## License

By contributing, you agree that your contributions are licensed under the MIT License.
