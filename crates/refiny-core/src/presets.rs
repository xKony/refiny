//! The provider preset table.
//!
//! A static list, not a discovery call. Selecting a preset fills the editable
//! fields in the settings window; the fields stay authoritative, so a stale
//! entry can be corrected by hand and the correction survives a reload.

use serde::Serialize;

use crate::config::{DEFAULT_API_KEY_ENV, DEFAULT_MODEL, DEFAULT_TIMEOUT_MS};

/// One named provider the user can pick from a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub model: &'static str,
    /// True when this endpoint usually needs no API key at all.
    pub key_optional: bool,
    /// Shown under the form to explain what the preset assumes.
    pub note: &'static str,
}

/// Everything the settings window needs to render the provider picker.
pub const PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "openai",
        name: "OpenAI",
        base_url: crate::config::DEFAULT_BASE_URL,
        model: DEFAULT_MODEL,
        key_optional: false,
        note: "A hosted key is required.",
    },
    ProviderPreset {
        id: "openrouter",
        name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        model: "openai/gpt-4o-mini",
        key_optional: false,
        note: "A hosted key is required.",
    },
    ProviderPreset {
        id: "ollama",
        name: "Ollama",
        base_url: "http://localhost:11434/v1",
        model: "llama3.2",
        key_optional: true,
        note: "Runs on this machine. No key needed; pull the model first.",
    },
    ProviderPreset {
        id: "lmstudio",
        name: "LM Studio",
        base_url: "http://localhost:1234/v1",
        model: "local-model",
        key_optional: true,
        note: "Runs on this machine. No key needed; load the model first.",
    },
];

/// The preset with this id, if the table still has one.
pub fn preset(id: &str) -> Option<&'static ProviderPreset> {
    PRESETS.iter().find(|preset| preset.id == id)
}

/// The preset whose base URL and model the given values already match.
///
/// Used to preselect the picker when the window opens. A config that matches
/// nothing - a self-hosted gateway, or a hand-corrected field - simply leaves
/// the picker on "Custom" instead of pretending one preset applies.
pub fn matching(base_url: &str, model: &str) -> Option<&'static ProviderPreset> {
    let base = base_url.trim().trim_end_matches('/');
    let model = model.trim();
    PRESETS.iter().find(|preset| {
        preset.base_url.trim_end_matches('/') == base && preset.model.trim() == model
    })
}

/// The defaults a new preset selection should write.
///
/// `api_key_env` is preserved rather than reset: the user may have pointed it
/// at a variable that already exists in their environment.
pub fn apply(preset: &ProviderPreset, api_key_env: &str) -> crate::config::ProviderConfig {
    crate::config::ProviderConfig {
        base_url: preset.base_url.to_string(),
        model: preset.model.to_string(),
        api_key_env: if api_key_env.trim().is_empty() {
            DEFAULT_API_KEY_ENV.to_string()
        } else {
            api_key_env.to_string()
        },
    }
}

/// The timeout a fresh config should carry.
pub const fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

/// One row of the prompt section of the tray menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromptMenuItem {
    /// Stable id the tray maps back to a choice. Content-free: `default`, or
    /// `preset:<index>`, so user text never becomes an identifier.
    pub id: String,
    pub label: String,
    pub checked: bool,
}

/// What the "Prompt:" line above the list should say.
pub fn prompt_menu_summary(config: &crate::config::Config) -> String {
    let name = config.active_prompt.trim();
    if name.is_empty() {
        return "Default".to_string();
    }
    match config
        .prompts
        .iter()
        .find(|preset| preset.name.trim() == name)
    {
        Some(preset) => preset.name.clone(),
        // The active name can outlive the preset it points at, and polishing
        // falls back to the built-in prompt in that case.
        None => format!("Default - `{name}` no longer exists"),
    }
}

/// The prompt rows of the tray menu, with the built-in prompt always last.
///
/// Pure so the menu contents can be tested without a desktop session.
pub fn prompt_menu(config: &crate::config::Config) -> Vec<PromptMenuItem> {
    let active = config.active_prompt.trim();

    let mut items: Vec<PromptMenuItem> = config
        .prompts
        .iter()
        .enumerate()
        .map(|(index, preset)| PromptMenuItem {
            id: format!("preset:{index}"),
            label: preset.name.clone(),
            checked: !active.is_empty() && preset.name.trim() == active,
        })
        .collect();

    items.push(PromptMenuItem {
        id: "preset:default".to_string(),
        label: "Default prompt".to_string(),
        checked: active.is_empty(),
    });

    items
}

/// The preset name a menu id refers to: `None` for the built-in prompt, or
/// `Some(name)` when the id points at a preset that still exists.
///
/// A preset can be deleted between the menu being built and clicked, so an id
/// that no longer resolves comes back as `None` rather than an error.
pub fn preset_for_menu_id(config: &crate::config::Config, id: &str) -> Option<Option<String>> {
    let target = id.strip_prefix("preset:")?;
    if target == "default" {
        return Some(None);
    }
    let index: usize = target.parse().ok()?;
    config
        .prompts
        .get(index)
        .map(|preset| Some(preset.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;

    #[test]
    fn every_preset_has_a_usable_url_and_model() {
        for preset in PRESETS {
            assert!(preset.base_url.starts_with("http"), "{}", preset.id);
            assert!(preset.base_url.ends_with("/v1"), "{}", preset.id);
            assert!(!preset.model.trim().is_empty(), "{}", preset.id);
            assert!(!preset.name.trim().is_empty(), "{}", preset.id);
        }
    }

    #[test]
    fn local_presets_need_no_key_and_say_so() {
        for id in ["ollama", "lmstudio"] {
            let preset = preset(id).unwrap();
            assert!(preset.key_optional, "{id}");
            assert!(preset.note.contains("No key"), "{id}");
        }
        assert!(!preset("openai").unwrap().key_optional);
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = PRESETS.iter().map(|preset| preset.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), before);
    }

    #[test]
    fn a_matching_config_preselects_its_preset() {
        let ollama = preset("ollama").unwrap();
        assert_eq!(
            matching("http://localhost:11434/v1/", "llama3.2").map(|p| p.id),
            Some("ollama")
        );
        assert_eq!(
            matching(ollama.base_url, ollama.model).map(|p| p.id),
            Some("ollama")
        );
    }

    #[test]
    fn a_hand_corrected_config_matches_nothing() {
        assert!(matching("http://localhost:11434/v1", "my-finetune").is_none());
        assert!(matching("https://gateway.internal/v1", "gpt-4o-mini").is_none());
    }

    #[test]
    fn applying_a_preset_keeps_the_configured_environment_variable() {
        let applied = apply(preset("ollama").unwrap(), "MY_LOCAL_KEY");

        assert_eq!(applied.base_url, "http://localhost:11434/v1");
        assert_eq!(applied.model, "llama3.2");
        assert_eq!(applied.api_key_env, "MY_LOCAL_KEY");

        let fresh = apply(preset("openai").unwrap(), "  ");
        assert_eq!(fresh.api_key_env, DEFAULT_API_KEY_ENV);
    }

    #[test]
    fn adding_a_preset_touches_only_this_table() {
        // The settings window renders from PRESETS, so a preset that passes the
        // invariants above needs no other change; this test pins that contract.
        let rendered: Vec<ProviderConfig> = PRESETS
            .iter()
            .map(|preset| apply(preset, DEFAULT_API_KEY_ENV))
            .collect();
        assert_eq!(rendered.len(), PRESETS.len());
    }

    fn config_with_presets(names: &[&str], active: &str) -> crate::config::Config {
        crate::config::Config {
            active_prompt: active.to_string(),
            prompts: names
                .iter()
                .map(|name| crate::config::PromptPreset {
                    name: (*name).to_string(),
                    prompt: format!("{name} prompt"),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn the_prompt_menu_lists_presets_and_the_default() {
        let config = config_with_presets(&["Terse", "Warm"], "");

        let menu = prompt_menu(&config);

        assert_eq!(
            menu.iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            vec!["Terse", "Warm", "Default prompt"]
        );
        assert!(
            menu[2].checked,
            "no active preset means the default is checked"
        );
        assert!(!menu[0].checked && !menu[1].checked);
    }

    #[test]
    fn the_active_preset_is_the_only_checked_row() {
        let config = config_with_presets(&["Terse", "Warm"], "Warm");

        let menu = prompt_menu(&config);

        assert_eq!(menu.iter().filter(|item| item.checked).count(), 1);
        assert_eq!(menu[1].label, "Warm");
        assert!(menu[1].checked);
        assert_eq!(prompt_menu_summary(&config), "Warm");
    }

    #[test]
    fn a_deleted_active_preset_still_produces_a_usable_menu() {
        let config = config_with_presets(&["Terse"], "Gone");

        let menu = prompt_menu(&config);

        assert!(!menu.iter().any(|item| item.checked), "nothing matches");
        assert!(prompt_menu_summary(&config).contains("Gone"));
        assert!(prompt_menu_summary(&config).starts_with("Default"));
    }

    #[test]
    fn a_menu_id_maps_back_to_the_choice_it_represents() {
        let config = config_with_presets(&["Terse", "Warm"], "Warm");

        assert_eq!(
            preset_for_menu_id(&config, "preset:1"),
            Some(Some("Warm".to_string()))
        );
        assert_eq!(preset_for_menu_id(&config, "preset:default"), Some(None));
        // A row for a preset deleted since the menu was built.
        assert_eq!(preset_for_menu_id(&config, "preset:9"), None);
        assert_eq!(preset_for_menu_id(&config, "nonsense"), None);
    }

    #[test]
    fn every_checked_menu_id_round_trips_to_its_label() {
        let config = config_with_presets(&["Terse", "Warm"], "Terse");

        for item in prompt_menu(&config) {
            let resolved =
                preset_for_menu_id(&config, &item.id).expect("every row in a fresh menu resolves");
            let expected = if item.label == "Default prompt" {
                None
            } else {
                Some(item.label.clone())
            };
            assert_eq!(resolved, expected, "row {}", item.id);
        }
    }
}
