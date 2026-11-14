# ADR-0006: Store the API key in the OS keychain

- Status: Accepted
- Date: 2026-11-14
- Deciders: xKony

## Context

ADR-0003 and the whitepaper both promise that secrets never live in `config.toml`, and until now the only way to supply a key was an environment variable. That promise is easy to keep when there is no settings UI, but it collides with the goal of configuring Refiny without leaving the app: an environment variable cannot be set from inside a running process, so the user must edit their shell, quit Refiny, and relaunch.

Three options were considered:

1. **Environment variable only.** Keeps the promise, keeps the code tiny, and cannot be done from the UI.
2. **Plaintext in `config.toml`.** Simplest to build, easy for the user to inspect and back up, and breaks the promise in three documents at once. A config file is also routinely copied, pasted into issues, and synced to cloud storage.
3. **OS keychain** (Windows Credential Manager, macOS Keychain, libsecret on Linux) via a Rust keyring crate, with a paste field in the settings UI.

## Decision

Store the API key in the OS keychain through the `keyring` crate, and accept a key pasted into the settings window as a first-class input. The environment variable named by `provider.api_key_env` remains a supported fallback for headless and scripted setups, and it is consulted when no key is stored in the keychain.

`config.toml` continues to hold only non-secret values.

## Consequences

- A user can supply a key without a shell command and without a restart, which is what makes the app usable on first run.
- The key is protected by the platform's user-scoped credential store rather than by file permissions alone.
- Adds a dependency with per-platform backends, and a Linux system requirement: libsecret, plus a running Secret Service. Minimal Linux desktop installs and headless sessions may not have one, which is why the environment-variable fallback stays.
- The keychain is not available in CI, so the secret store must sit behind a seam that lets tests substitute an in-memory implementation.
- Docs must stop saying the environment variable is the only way to supply a key, while still recommending it where a key already lives in the environment.
- A user who wants to move settings between machines still has to re-enter the key; that is the intended cost of not writing it to a file.
