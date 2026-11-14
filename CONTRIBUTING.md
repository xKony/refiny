# Contributing to Refiny

Refiny is pre-alpha, version 0.0.1, with one author and one contributor. It has no releases and no published installers. Work is tracked in GitHub issues.

## Prerequisites

- **Rust stable**, edition 2024. `rust-toolchain.toml` pins the stable channel with `rustfmt` and `clippy`.
- **Tauri 2 CLI**, for `cargo tauri dev`: `cargo install tauri-cli --version "^2"`.
- **Platform packages:**
  - **Windows:** WebView2 (preinstalled on Windows 11) and the MSVC build tools.
  - **macOS:** Xcode Command Line Tools.
  - **Linux:** `libwebkit2gtk-4.1-dev`, `libappindicator3-dev` (or `libayatana-appindicator3-dev`), `librsvg2-dev`, `libgtk-3-dev`, `libx11-dev`, and `patchelf`.

Only Windows has been run. The macOS and Linux lists are what compilation requires, not a statement that the app works there; see the platform status in the [README](README.md#platform-status).

## Build and test

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p refiny-core
cargo tauri dev
```

CI runs `cargo fmt --all -- --check` and `cargo test -p refiny-core` on Windows, macOS, and Ubuntu, and `cargo check -p refiny` plus clippy on the same three. A green run proves the code compiles and the core tests pass. It does not execute the app.

## Where code goes

- `crates/refiny-core` - configuration, prompt construction, provider client. Keep Tauri, window, clipboard, and input code out of it.
- `src-tauri` - the Tauri shell. Platform-specific behavior belongs in `src-tauri/src/selection.rs`, the one boundary between the app and the OS.
- `src` - static HTML, CSS, and JS. No bundler and no Node toolchain.

What each module owns is in [docs/architecture.md](docs/architecture.md#workspace-layout).

## Testing

The current test inventory, and what is not covered, is in [docs/architecture.md](docs/architecture.md#testing).

- Add a unit test for any change to the prompt builder, config loader, or provider client.
- `src-tauri` has no tests and no seam that would let the polish action be tested with a fake provider or a fake selection service. If you change its behavior, say in the issue what you ran by hand and on which platform. Building that seam is [issue #4](https://github.com/xKony/refiny/issues/4).
- Do not claim platform verification you did not perform.

## Configuration during development

Point `provider.base_url` at a local OpenAI-compatible server such as Ollama or LM Studio to avoid a hosted provider, and set the environment variable named by `provider.api_key_env`. The file's location, format, and read-at-startup behavior are in the [README](README.md#configuration).

Never commit API keys, tokens, or private text samples.

## Code style

- Run `cargo fmt --all` before committing.
- Clippy warnings are errors in CI.
- Prefer small typed errors over stringly-typed failures in `refiny-core`.
- The prompt in `crates/refiny-core/src/prompt.rs` is authoritative. If you change it, update the contract summary in the [whitepaper](docs/whitepaper.md#3-the-transformation-contract) to match.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(selection): restore the clipboard after a failed paste
fix(tray): raise the settings window instead of toggling it
docs: correct the platform status table
```

## Issues and pull requests

- Bug and feature issues use the GitHub templates. Triage labels are documented in [docs/agents/triage-labels.md](docs/agents/triage-labels.md).
- One logical change per pull request, with the tests and documentation it needs.
- Describe the user-visible behavior and the failure modes you considered.
- Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).

Security problems go through [SECURITY.md](SECURITY.md), not a public issue.

## License

By contributing, you agree that your contributions are licensed under the MIT License. See [LICENSE](LICENSE).
