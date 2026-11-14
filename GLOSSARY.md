# Glossary

The vocabulary Refiny uses in code, issues, and documentation. Prefer these terms over synonyms.

| Term | Definition | Avoid |
| --- | --- | --- |
| **Refiny** | The project and the desktop application. | Refiner, Refinery |
| **Polish** | The transformation from rough text to corrected text. | Refine (as a noun), rewrite |
| **Polish action** | One hotkey-triggered run: capture → polish → replace. | Job, task |
| **Selection** | The text highlighted in the foreground app when the hotkey fires. | Highlight |
| **Capture** | Reading the selection through a clipboard round-trip. | Grab, copy |
| **Replace** | Writing polished text over the selection with a synthetic paste. | Insert, overwrite |
| **Selection seam** | The `Selection` trait in `refiny-core`: what a polish action needs from the OS, implemented by `OsSelection` in the shell and by fakes in tests. | Clipboard abstraction |
| **Preservation list** | User-defined terms that must remain byte-identical. | Whitelist, dictionary |
| **Prompt template** | The prompt text used for a polish. The default one is user-editable; empty means the built-in default. | Custom prompt |
| **Prompt preset** | A named prompt template the user can switch to from the tray. | Profile, mode |
| **Active prompt** | The preset named by `active_prompt`, or the default template when that name is empty or unknown. | Current prompt |
| **Provider** | Any OpenAI-compatible chat-completions endpoint. | Backend, API |
| **Provider preset** | A named entry in the static table that fills the base URL and default model in the settings window. | Template, profile |
| **Config store** | The `ConfigStore` seam plus `LiveConfig`, which owns the current configuration and writes it. | Settings manager |
| **Live config** | The configuration in effect right now. Reads go through it, so a save is visible to the next action. | Config (bare) |
| **Secret store** | The `SecretStore` seam holding the API key outside `config.toml`; the OS keychain in the app, in memory in tests. | Vault |
| **Key source** | Where the key in effect came from: `keychain`, `environment`, or `none`. Shown in the status bar. | |
| **Overlay** | Transparent always-on-top window shown during a polish action. | Popup, HUD |
| **Tray** | System status-area icon that keeps Refiny running when the settings window is hidden. | Notification area, menu bar icon |
| **Mascot** | The animated low-poly character in the overlay. | Avatar, spinner |
| **Hotkey** | System-wide keyboard shortcut that starts a polish action. | Shortcut, keybind |
| **Playground** | The settings tab that sends typed text through the same pipeline as the hotkey, without touching the selection. | Try the pipeline, test box |
| **Mixed-language input** | Text containing spans in more than one natural language; each span keeps its language. | Multilingual mode |
