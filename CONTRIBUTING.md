# Contributing to Refiny

Refiny is pre-alpha, version 0.0.1, with one author and one contributor. It has no releases and no published installers. Work is tracked in GitHub issues.

Small, focused changes are easiest to review. Open an issue first if the change spans more than one module or alters the pipeline.

## Prerequisites

- **Rust stable**, edition 2024. `rust-toolchain.toml` pins the stable channel with `rustfmt` and `clippy`.
- **Tauri 2 CLI**, for `cargo tauri dev`: `cargo install tauri-cli --version "^2"`.
- **Platform packages:**
  - **Windows:** WebView2 (preinstalled on Windows 11) and the MSVC build tools.
  - **macOS:** Xcode Command Line Tools.
  - **Linux:** `libwebkit2gtk-4.1-dev`, `libappindicator3-dev` (or `libayatana-appindicator3-dev`), `librsvg2-dev`, `libgtk-3-dev`, `libx11-dev`, and `patchelf`. These are the packages CI installs on Ubuntu.

## Build and test

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p refiny-core
cargo tauri dev
```

Those commands cover every check CI makes. CI runs `cargo fmt --all -- --check` and `cargo test -p refiny-core` on Windows, macOS, and Ubuntu, then `cargo check -p refiny` and clippy on the same three. `refiny-core` has no UI or OS dependencies, so its tests are the fastest feedback loop and the only tests that exist.

Changing the app icon requires the source PNG and the Tauri CLI:

```bash
cargo tauri icon src-tauri/icons/icon.png
```

## Where code goes

- `crates/refiny-core` - configuration, prompt construction, provider client. Keep Tauri, window, clipboard, and input code out of it.
- `src-tauri` - the Tauri shell. Platform-specific behavior belongs in `src-tauri/src/selection.rs`, the one boundary between the app and the OS.
- `src` - static HTML, CSS, and JS. No bundler and no Node toolchain.

## Testing expectations

There are 7 unit tests, all in `refiny-core`, covering config load and save, prompt assembly, and the provider client against an HTTP mock.

- Add a unit test for any change to the prompt builder, config loader, or provider client.
- `src-tauri` has no tests and no seam that would make the polish action testable with a fake provider or a fake selection service. If you change its behavior, say in the issue what you ran by hand and on which platform.
- Do not claim platform verification you did not perform. The author has run the app on Windows only.

## Configuration during development

Refiny reads `config.toml` from Tauri's app config directory. On Windows that is `%APPDATA%\dev.refiny.app\config.toml`; the settings window displays the resolved path. For local experiments, point `provider.base_url` at a local OpenAI-compatible server such as Ollama or LM Studio and set the environment variable named by `provider.api_key_env`.

The app reads this file at startup and never writes it. Restart Refiny after editing it.

Never commit API keys, tokens, or private text samples.

## Code style

- Run `cargo fmt --all` before committing.
- Clippy warnings are errors in CI.
- Prefer small typed errors over stringly-typed failures in `refiny-core`.
- Keep the default prompt in `crates/refiny-core/src/prompt.rs` and the transformation contract in the [whitepaper](docs/whitepaper.md) in sync.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(core): add per-app prompt profiles
fix(selection): restore clipboard after a failed paste
docs: clarify Wayland limitations
```

## Issues and pull requests

- Bug and feature issues use the GitHub templates. Triage labels are documented in [docs/agents/triage-labels.md](docs/agents/triage-labels.md).
- One logical change per pull request, with the tests and documentation it needs.
- Describe the user-visible behavior and the failure modes you considered.
- Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).

Security problems go through [SECURITY.md](SECURITY.md), not a public issue.

## License

By contributing, you agree that your contributions are licensed under the MIT License. See [LICENSE](LICENSE).
