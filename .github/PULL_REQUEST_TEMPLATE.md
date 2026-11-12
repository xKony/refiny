## What changed

<!-- One or two sentences. Link the issue this closes, if any. -->

Closes #

## Why

<!-- The user-visible problem this solves. -->

## How to test

<!-- Steps a reviewer can follow, including the platform and foreground app. -->

## Checklist

- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test -p refiny-core` passes
- [ ] Behavior changes have tests (prompt builder, config, provider)
- [ ] Docs updated if the contract, config, or platform matrix changed
- [ ] No API keys, tokens, or private text in the diff
