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

CI runs two jobs on Windows, macOS, and Ubuntu. One runs `cargo fmt --all -- --check`, `cargo clippy -p refiny-core --all-targets -- -D warnings`, and `cargo test -p refiny-core`. The other runs `cargo check -p refiny --locked` and `cargo clippy -p refiny --all-targets -- -D warnings`, installing the Linux system packages first on Ubuntu. A green run proves the code compiles and the core tests pass. It does not execute the app.

## Where code goes

- `crates/refiny-core` - configuration and its store, the secret store seam, prompt construction, the selection seam, the polish action's orchestration, provider presets, and the provider client. Keep Tauri, window, clipboard, and input code out of it.
- `src-tauri` - the Tauri shell. The two OS boundaries are `src-tauri/src/selection.rs` (clipboard round trip and synthetic input) and `src-tauri/src/keychain.rs` (the OS credential store). Both implement traits from `refiny-core`.
- `src` - static HTML, CSS, and JS. No bundler and no Node toolchain.

When something in the app needs the OS, define the trait in `refiny-core`, implement it in `src-tauri`, and add an in-memory implementation for tests. That is what makes the polish action testable without a desktop session.

What each module owns is in [docs/architecture.md](docs/architecture.md#workspace-layout).

## Testing

The current test inventory, and what is not covered, is in [docs/architecture.md](docs/architecture.md#testing).

- Add a unit test for any change to the prompt builder, config layer, secret resolution, preset table, orchestration, or provider client.
- Tests create scratch space under `target/test-tmp` through `crates/refiny-core/src/test_support.rs`, not the machine temp directory, so they do not depend on `TMPDIR` permissions. Use `test_support::temp_dir()` rather than `tempfile::tempdir()`.
- Anything in `src-tauri` that cannot be tested without a desktop session: say in the issue what you ran by hand and on which platform.
- Do not claim platform verification you did not perform.

## Configuration during development

Point the provider at a local OpenAI-compatible server such as Ollama or LM Studio to avoid a hosted provider. The settings window is the fastest way: pick a preset, edit the model if yours differs, and save.

During development the app writes its config to the platform's app config directory. To keep runs isolated, or to run without touching your real settings, override the directory at build time:

```bash
TAURI_CONFIG='{"app":{"appDirectoriesOverride":"./dev-data"}}' cargo tauri dev
```

The file's location, format, and behavior are in the [README](README.md#configuration). The API key is not in it; it goes to the OS credential store, which is why `cargo test -p refiny` keeps the real-keychain test `#[ignore]`d.

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
