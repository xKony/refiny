const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;

const SETUP_DISMISSED_KEY = "refiny-setup-dismissed";

const els = {
  setup: document.querySelector("#setup"),
  setupTitle: document.querySelector("#setup-title"),
  setupStart: document.querySelector("#setup-start"),
  setupStatus: document.querySelector("#setup-status"),
  dismissSetup: document.querySelector("#dismiss-setup"),

  hotkey: document.querySelector("#hotkey"),
  provider: document.querySelector("#provider"),
  apiKey: document.querySelector("#api-key"),
  activePrompt: document.querySelector("#active-prompt"),
  footerPath: document.querySelector("#footer-path"),
  lastAction: document.querySelector("#last-action"),
  version: document.querySelector("#version"),

  tabs: [...document.querySelectorAll(".tab")],

  presets: document.querySelector("#presets"),
  providerForm: document.querySelector("#provider-form"),
  baseUrl: document.querySelector("#base-url"),
  model: document.querySelector("#model"),
  apiKeyInput: document.querySelector("#api-key-input"),
  apiKeyEnv: document.querySelector("#api-key-env"),
  keyNote: document.querySelector("#key-note"),
  saveProvider: document.querySelector("#save-provider"),
  testProvider: document.querySelector("#test-provider"),
  clearKey: document.querySelector("#clear-key"),
  providerStatus: document.querySelector("#provider-status"),

  hotkeyForm: document.querySelector("#hotkey-form"),
  hotkeyInput: document.querySelector("#hotkey-input"),
  saveHotkey: document.querySelector("#save-hotkey"),
  hotkeyStatus: document.querySelector("#hotkey-status"),

  promptForm: document.querySelector("#prompt-form"),
  promptInput: document.querySelector("#prompt-input"),
  preserveInput: document.querySelector("#preserve-input"),
  presetList: document.querySelector("#preset-list"),
  addPreset: document.querySelector("#add-preset"),
  savePrompts: document.querySelector("#save-prompts"),
  promptStatus: document.querySelector("#prompt-status"),

  input: document.querySelector("#input"),
  polish: document.querySelector("#polish"),
  output: document.querySelector("#output"),
  outputEmpty: document.querySelector("#output-empty"),
  status: document.querySelector("#result-status"),
};

const isMac = /Mac/.test(navigator.userAgent);
const TAB_KEY = "refiny-active-tab";

/** The config last received from the backend. */
let current = null;
/** Provider presets, sent by the backend so the table has one home. */
let presets = [];
/** Preset names and prompts as the user is editing them, saved on submit. */
let draftPresets = [];

function prettyHotkey(hotkey) {
  return hotkey.replace("CommandOrControl", isMac ? "Cmd" : "Ctrl");
}

function keySourceLabel(source) {
  if (source === "keychain") return "in keychain";
  if (source === "environment") return "from environment";
  return "missing";
}

function setStatus(el, message, isError = false) {
  el.textContent = message;
  el.classList.toggle("error", isError);
  el.classList.toggle("ok", !isError && Boolean(message));
}

function setBusy(button, busy) {
  button.disabled = busy;
}

/* Tabs ------------------------------------------------------------------- */

function showTab(panelId, { focus = false } = {}) {
  for (const tab of els.tabs) {
    const selected = tab.dataset.panel === panelId;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    tab.classList.toggle("active", selected);
    document.querySelector(`#${tab.dataset.panel}`).hidden = !selected;
    if (selected && focus) tab.focus();
  }
  localStorage.setItem(TAB_KEY, panelId);
}

for (const tab of els.tabs) {
  tab.addEventListener("click", () => showTab(tab.dataset.panel));
  tab.addEventListener("keydown", (event) => {
    const index = els.tabs.indexOf(tab);
    if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
      event.preventDefault();
      const delta = event.key === "ArrowRight" ? 1 : -1;
      const next = els.tabs[(index + delta + els.tabs.length) % els.tabs.length];
      showTab(next.dataset.panel, { focus: true });
    }
  });
}

function restoreTab() {
  const saved = localStorage.getItem(TAB_KEY);
  const valid = els.tabs.some((tab) => tab.dataset.panel === saved);
  showTab(valid ? saved : "panel-provider");
}

/** Setup has not been finished, so land on the section that finishes it. */
function focusFirstRun() {
  showTab("panel-provider");
  els.baseUrl.focus();
}

/* Provider presets ------------------------------------------------------- */

function renderPresets() {
  els.presets.replaceChildren();
  for (const preset of presets) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "preset";
    button.dataset.presetId = preset.id;
    button.setAttribute("role", "radio");
    button.setAttribute("aria-checked", "false");

    const name = document.createElement("strong");
    name.textContent = preset.name;
    const meta = document.createElement("span");
    meta.textContent = preset.key_optional ? "No key needed" : "Key required";
    button.append(name, meta);

    button.addEventListener("click", () => applyPreset(preset));
    els.presets.append(button);
  }
  markActivePreset();
}

function markActivePreset() {
  const matched = current?.matched_preset ?? null;
  for (const button of els.presets.querySelectorAll(".preset")) {
    const active = button.dataset.presetId === matched;
    button.classList.toggle("selected", active);
    button.setAttribute("aria-checked", String(active));
  }
}

function applyPreset(preset) {
  els.baseUrl.value = preset.base_url;
  els.model.value = preset.model;
  // The fields stay authoritative: selecting a preset only fills them.
  current = { ...current, matched_preset: preset.id };
  markActivePreset();
  updateKeyNote();
  setStatus(els.providerStatus, `${preset.name} filled in. Save to apply.`);
}

/* Loading ---------------------------------------------------------------- */

function applyConfig(config) {
  current = config;
  presets = config.presets ?? [];

  els.version.textContent = config.version;
  els.hotkey.textContent = prettyHotkey(config.hotkey);
  els.provider.textContent = config.model;
  els.provider.title = config.base_url;
  els.apiKey.textContent = keySourceLabel(config.key_source);
  els.apiKey.classList.toggle("warn", !config.api_key_present);
  els.activePrompt.textContent = config.active_prompt?.trim() || "Default";
  els.footerPath.textContent = config.config_path;
  els.footerPath.title = config.config_path;

  // Fields show the values in effect, so the user can confirm a save.
  els.baseUrl.value = config.base_url;
  els.model.value = config.model;
  els.apiKeyEnv.value = config.api_key_env;
  els.apiKeyInput.value = "";
  els.hotkeyInput.value = config.hotkey;

  els.promptInput.value = config.prompt ?? "";
  els.promptInput.placeholder = config.prompt
    ? ""
    : "Empty means the built-in copy-editing prompt. Type here to replace it.";
  els.preserveInput.value = (config.preserve ?? []).join("\n");
  draftPresets = (config.prompts ?? []).map((preset) => ({ ...preset }));
  renderPresetEditor();

  updateKeyNote();
  renderPresets();
  updateSetupPanel();
}

function updateKeyNote() {
  const preset = presets.find((entry) => entry.id === current?.matched_preset);
  els.keyNote.textContent = preset
    ? preset.note
    : "A key is optional for a local server. Leaving the field blank keeps whatever is stored.";
  els.clearKey.hidden = current?.key_source !== "keychain";
}

function updateSetupPanel() {
  const dismissed = localStorage.getItem(SETUP_DISMISSED_KEY) === "1";
  const showSetup = !current?.setup_complete && !dismissed;
  els.setup.hidden = !showSetup;
  if (showSetup) {
    els.setupTitle.textContent = current?.first_run ? "Welcome to Refiny" : "Finish setup";
  }
}

async function loadConfig() {
  if (!invoke) {
    els.footerPath.textContent = "run via cargo tauri dev";
    return;
  }

  try {
    const config = await invoke("get_config");
    applyConfig(config);
    // A remembered tab is the right default, except on a fresh install where
    // the only useful place to start is the provider form.
    if (!config.setup_complete) focusFirstRun();
  } catch (error) {
    setStatus(els.providerStatus, String(error), true);
  }
}

/* Provider form ---------------------------------------------------------- */

async function saveProvider(event) {
  event?.preventDefault();
  if (!invoke) return;

  setBusy(els.saveProvider, true);
  setStatus(els.providerStatus, "Saving…");

  try {
    const config = await invoke("save_provider", {
      form: {
        base_url: els.baseUrl.value,
        model: els.model.value,
        api_key_env: els.apiKeyEnv.value,
        api_key: els.apiKeyInput.value.trim() || null,
        clear_api_key: false,
      },
    });
    applyConfig(config);
    setStatus(els.providerStatus, "Saved. The next action uses it.");
  } catch (error) {
    setStatus(els.providerStatus, String(error), true);
  } finally {
    setBusy(els.saveProvider, false);
  }
}

async function testProvider() {
  if (!invoke) return;

  setBusy(els.testProvider, true);
  setStatus(els.providerStatus, "Testing…");

  try {
    const result = await invoke("test_provider", {
      form: {
        base_url: els.baseUrl.value,
        model: els.model.value,
        api_key: els.apiKeyInput.value.trim() || null,
      },
    });
    setStatus(els.providerStatus, result.message, !result.ok);
  } catch (error) {
    setStatus(els.providerStatus, String(error), true);
  } finally {
    setBusy(els.testProvider, false);
  }
}

async function clearStoredKey() {
  if (!invoke) return;

  setBusy(els.clearKey, true);
  try {
    const config = await invoke("save_provider", {
      form: {
        base_url: els.baseUrl.value,
        model: els.model.value,
        api_key_env: els.apiKeyEnv.value,
        api_key: null,
        clear_api_key: true,
      },
    });
    applyConfig(config);
    setStatus(els.providerStatus, "Stored key removed.");
  } catch (error) {
    setStatus(els.providerStatus, String(error), true);
  } finally {
    setBusy(els.clearKey, false);
  }
}

/* Hotkey form ------------------------------------------------------------ */

async function saveHotkey(event) {
  event?.preventDefault();
  if (!invoke) return;

  setBusy(els.saveHotkey, true);
  setStatus(els.hotkeyStatus, "Applying…");

  try {
    const config = await invoke("save_hotkey", { form: { hotkey: els.hotkeyInput.value } });
    applyConfig(config);
    setStatus(els.hotkeyStatus, "Hotkey is live.");
  } catch (error) {
    // The previous hotkey is still registered; put its value back in the field
    // so the form matches what actually works.
    els.hotkeyInput.value = current?.hotkey ?? els.hotkeyInput.value;
    setStatus(els.hotkeyStatus, String(error), true);
  } finally {
    setBusy(els.saveHotkey, false);
  }
}

/* Prompt form ------------------------------------------------------------ */

function renderPresetEditor() {
  els.presetList.replaceChildren();

  if (draftPresets.length === 0) {
    const empty = document.createElement("p");
    empty.className = "hint empty-note";
    empty.textContent = "No named presets yet. Add one to switch prompts from the tray.";
    els.presetList.append(empty);
    return;
  }

  draftPresets.forEach((preset, index) => {
    const row = document.createElement("div");
    row.className = "preset-row";

    const name = document.createElement("input");
    name.type = "text";
    name.value = preset.name;
    name.placeholder = "Preset name";
    name.spellcheck = false;
    name.setAttribute("aria-label", `Preset ${index + 1} name`);
    name.addEventListener("input", () => {
      draftPresets[index].name = name.value;
    });

    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "link-button danger-link";
    remove.textContent = "Remove";
    remove.setAttribute("aria-label", `Remove preset ${index + 1}`);
    remove.addEventListener("click", () => {
      draftPresets.splice(index, 1);
      renderPresetEditor();
    });

    const head = document.createElement("div");
    head.className = "preset-row-head";
    head.append(name, remove);

    const prompt = document.createElement("textarea");
    prompt.rows = 2;
    prompt.value = preset.prompt;
    prompt.placeholder = "System prompt for this preset";
    prompt.spellcheck = false;
    prompt.setAttribute("aria-label", `Preset ${index + 1} prompt`);
    prompt.addEventListener("input", () => {
      draftPresets[index].prompt = prompt.value;
    });

    row.append(head, prompt);
    els.presetList.append(row);
  });
}

async function savePrompts(event) {
  event?.preventDefault();
  if (!invoke) return;

  setBusy(els.savePrompts, true);
  setStatus(els.promptStatus, "Saving…");

  try {
    const config = await invoke("save_prompts", {
      form: {
        prompt: els.promptInput.value,
        // Blank lines are what an empty textarea produces; they are not terms.
        preserve: els.preserveInput.value
          .split("\n")
          .map((term) => term.trim())
          .filter(Boolean),
        prompts: draftPresets.map((preset) => ({
          name: preset.name.trim(),
          prompt: preset.prompt,
        })),
        active_prompt: current?.active_prompt ?? "",
      },
    });
    applyConfig(config);
    setStatus(els.promptStatus, "Saved.");
  } catch (error) {
    setStatus(els.promptStatus, String(error), true);
  } finally {
    setBusy(els.savePrompts, false);
  }
}

/* Playground ------------------------------------------------------------- */

async function polish() {
  const text = els.input.value.trim();
  if (!text) {
    setStatus(els.status, "Type something first.", true);
    return;
  }
  if (!invoke) {
    setStatus(els.status, "Provider calls require the Refiny app.", true);
    return;
  }

  setBusy(els.polish, true);
  els.output.hidden = true;
  els.outputEmpty.hidden = true;
  setStatus(els.status, "Polishing…");

  try {
    const result = await invoke("polish_text", { text });
    els.output.textContent = result;
    els.output.hidden = false;
    setStatus(els.status, "Done.");
  } catch (error) {
    els.outputEmpty.hidden = false;
    setStatus(els.status, String(error), true);
  } finally {
    setBusy(els.polish, false);
  }
}

/* Wiring ----------------------------------------------------------------- */

// The action bar is outside each form so it stays visible while the panel
// scrolls, which means the buttons submit or trigger explicitly.
els.providerForm.addEventListener("submit", (event) => {
  event.preventDefault();
  saveProvider();
});
els.saveProvider.addEventListener("click", saveProvider);
els.testProvider.addEventListener("click", testProvider);
els.clearKey.addEventListener("click", clearStoredKey);

els.hotkeyForm.addEventListener("submit", (event) => {
  event.preventDefault();
  saveHotkey();
});
els.saveHotkey.addEventListener("click", saveHotkey);

els.promptForm.addEventListener("submit", (event) => {
  event.preventDefault();
  savePrompts();
});
els.savePrompts.addEventListener("click", savePrompts);
els.addPreset.addEventListener("click", () => {
  draftPresets.push({ name: "", prompt: "" });
  renderPresetEditor();
  els.presetList.querySelector(".preset-row:last-child input")?.focus();
});

// Enter in a single-line field of these forms saves, as a user expects.
for (const input of [els.baseUrl, els.model, els.apiKeyEnv, els.apiKeyInput]) {
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      saveProvider();
    }
  });
}
els.hotkeyInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    saveHotkey();
  }
});

els.setupStart.addEventListener("click", () => {
  els.setup.hidden = true;
  showTab("panel-provider");
  els.baseUrl.focus();
});
els.dismissSetup.addEventListener("click", () => {
  localStorage.setItem(SETUP_DISMISSED_KEY, "1");
  els.setup.hidden = true;
});

els.polish.addEventListener("click", polish);
els.input.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") polish();
});

// The tray can switch the active prompt while this window is open.
listen?.("refiny-config-changed", () => loadConfig());

listen?.("refiny-state", (event) => {
  const payload = event.payload ?? {};
  // The overlay owns the in-place feedback. This row is a history of the last
  // hotkey action, kept separate from the playground status so one never
  // overwrites the other.
  if (payload.state === "working") return;
  els.lastAction.textContent = payload.message || payload.state;
  els.lastAction.classList.toggle("warn", payload.state === "error");
});

restoreTab();
loadConfig();
