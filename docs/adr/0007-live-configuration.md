# ADR-0007: Live configuration behind a store seam

- Status: Accepted
- Date: 2026-11-14
- Deciders: xKony

## Context

Until now `AppState` held one `Config` value, cloned out of the file during setup. Nothing wrote the file after the first run, no command could change a setting, and `Config::save` was dead code exercised only by a test. Every adjustment - a provider URL, a model name, a hotkey - meant editing TOML, quitting from the tray, and relaunching.

The settings window makes that untenable. A form whose save does not take effect is worse than no form, and the tray's prompt presets need a setting that changes while the app runs.

Three shapes were available:

1. **Keep the value and add commands that mutate a copy in a `Mutex<Config>`.** Smallest diff. Leaves the persistence story implicit, and puts the "write it, and if the write fails undo the memory change" logic at each command.
2. **A store interface plus a live value.** One owner of load, save, and reload; commands ask it to apply an edit; tests substitute an in-memory implementation.
3. **Read the file on every access.** Always current, but turns a provider call into file I/O and makes a partially-written file a live hazard mid-action.

## Decision

Option 2. `LiveConfig` in `crates/refiny-core/src/config_store.rs` owns a `ConfigStore` and the current value behind an `RwLock`.

- `snapshot` is the read path, cheap and lock-scoped, so a caller takes a value and releases the lock before making a network call.
- `save` persists first and only then replaces the in-memory value. A failed write leaves the app running on values that match the file.
- `update` applies a closure to a copy and saves the result. An edit is all-or-nothing, and a value `Config::validate` rejects never reaches either the file or the running app.
- The app holds `Arc<LiveConfig>` and never a `Config`. Commands and polish actions read through it.
- `FileConfigStore` writes to a sibling temporary file and renames it over the target, so an interrupted write cannot leave a truncated config. On Windows a rename onto a file another process holds open falls back to a direct write rather than losing the save.

## Consequences

- A save is visible to the next polish action, which is what makes the settings window honest.
- Configuration logic is testable without touching the real user profile: `MemoryConfigStore` is a few lines and every store behaviour has a test against it.
- Commands compose edits instead of assigning fields, so a command cannot half-apply a change.
- The value in effect can drift from the file if something else edits it. `LiveConfig::reload` exists for that, and nothing calls it: there is no file watcher, so a hand edit still needs a restart. That is a deliberate gap, not an oversight - watching the file would also mean deciding what a mid-action reload does.
- Provider calls clone a `Config` per action. That is a handful of strings and a `Vec`, and it buys the property that an action cannot observe a config change halfway through.
