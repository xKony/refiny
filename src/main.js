const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;

const SETUP_DISMISSED_KEY = "refiny-setup-dismissed";

const els = {
  hotkey: document.querySelector("#hotkey"),
  provider: document.querySelector("#provider"),
  apiKey: document.querySelector("#api-key"),
  configPath: document.querySelector("#config-path"),
  setup: document.querySelector("#setup"),
  setupTitle: document.querySelector("#setup-title"),
  setupCommand: document.querySelector("#setup-command"),
  setupHotkey: document.querySelector("#setup-hotkey"),
  dismissSetup: document.querySelector("#dismiss-setup"),
  recheck: document.querySelector("#recheck"),
  setupStatus: document.querySelector("#setup-status"),
  input: document.querySelector("#input"),
  polish: document.querySelector("#polish"),
  output: document.querySelector("#output"),
  status: document.querySelector("#result-status"),
  version: document.querySelector("#version"),
};

const isMac = /Mac/.test(navigator.userAgent);
const isWindows = /Windows/.test(navigator.userAgent);

function prettyHotkey(hotkey) {
  return hotkey.replace("CommandOrControl", isMac ? "Cmd" : "Ctrl");
}

function setupCommand(envName) {
  if (isWindows) {
    return `setx ${envName} "sk-..."`;
  }
  if (isMac) {
    return `launchctl setenv ${envName} "sk-..."`;
  }
  return `export ${envName}="sk-..."`;
}

function setStatus(message, isError = false) {
  els.status.textContent = message;
  els.status.classList.toggle("error", isError);
}

function setSetupStatus(message, isError = false) {
  els.setupStatus.textContent = message;
  els.setupStatus.classList.toggle("error", isError);
}

function applyConfig(config) {
  els.hotkey.textContent = prettyHotkey(config.hotkey);
  els.provider.textContent = `${config.model} · ${config.base_url}`;
  els.apiKey.textContent = config.api_key_present
    ? `set via ${config.api_key_env}`
    : `missing ${config.api_key_env}`;
  els.apiKey.classList.toggle("warn", !config.api_key_present);
  els.configPath.textContent = config.config_path;
  els.version.textContent = config.version;

  const dismissed = localStorage.getItem(SETUP_DISMISSED_KEY) === "1";
  const showSetup = !config.api_key_present && !dismissed;
  els.setup.hidden = !showSetup;
  if (showSetup) {
    els.setupTitle.textContent = config.first_run ? "Welcome to Refiny" : "Finish setup";
    els.setupCommand.textContent = setupCommand(config.api_key_env);
    els.setupHotkey.textContent = prettyHotkey(config.hotkey);
  }
}

async function loadConfig() {
  if (!invoke) {
    els.configPath.textContent = "Run the app with `cargo tauri dev` to load configuration.";
    return;
  }

  try {
    const config = await invoke("get_config");
    applyConfig(config);
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function polish() {
  const text = els.input.value.trim();
  if (!text) {
    setStatus("Type something first.", true);
    return;
  }
  if (!invoke) {
    setStatus("Provider calls require the Refiny app.", true);
    return;
  }

  els.polish.disabled = true;
  els.output.hidden = true;
  setStatus("Polishing…");

  try {
    const result = await invoke("polish_text", { text });
    els.output.textContent = result;
    els.output.hidden = false;
    setStatus("Done.");
  } catch (error) {
    setStatus(String(error), true);
  } finally {
    els.polish.disabled = false;
  }
}

async function recheck() {
  if (!invoke) {
    return;
  }

  els.recheck.disabled = true;
  setSetupStatus("Checking…");

  try {
    const config = await invoke("get_config");
    applyConfig(config);
    if (config.api_key_present) {
      setSetupStatus("API key found.");
    } else {
      setSetupStatus(`Still missing ${config.api_key_env}. Restart Refiny after setting it.`, true);
    }
  } catch (error) {
    setSetupStatus(String(error), true);
  } finally {
    els.recheck.disabled = false;
  }
}

els.polish.addEventListener("click", polish);
els.input.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    polish();
  }
});

els.recheck.addEventListener("click", recheck);
els.dismissSetup.addEventListener("click", () => {
  localStorage.setItem(SETUP_DISMISSED_KEY, "1");
  els.setup.hidden = true;
});

listen?.("refiny-state", (event) => {
  const payload = event.payload ?? {};
  if (payload.state === "error") {
    setStatus(payload.message, true);
  }
});

loadConfig();
