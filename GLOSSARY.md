# Glossary

The vocabulary Refiny uses in code, issues, and documentation. Prefer these terms over synonyms.

| Term | Definition | Avoid |
| --- | --- | --- |
| **Refiny** | The project and the desktop application. | Refiner, Refinery |
| **Polish** | The transformation from rough text to corrected text. | Refine (as a noun), rewrite |
| **Polish action** | One hotkey-triggered run: capture → polish → replace. | Job, task |
| **Selection** | The text highlighted in the foreground app when the hotkey fires. | Highlight |
| **Capture** | Reading the selection, currently planned as a clipboard round-trip. | Grab, copy |
| **Replace** | Writing polished text over the selection, currently planned as a paste. | Insert, overwrite |
| **Preservation list** | User-defined terms that must remain byte-identical. | Whitelist, dictionary |
| **Prompt template** | User-editable system prompt; empty means the built-in default. | Custom prompt |
| **Mixed-language input** | Text containing spans in more than one natural language; each span keeps its language. | Multilingual mode |
| **Provider** | Any OpenAI-compatible chat-completions endpoint. | Backend, API |
| **Overlay** | Transparent always-on-top window shown during a polish action. | Popup, HUD |
| **Mascot** | The animated low-poly character in the overlay. | Avatar, spinner |
| **Hotkey** | System-wide keyboard shortcut that starts a polish action. | Shortcut, keybind |
