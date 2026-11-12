const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;

const els = {
  hotkey: document.querySelector("#hotkey"),
  provider: document.querySelector("#provider"),
  apiKey: document.querySelector("#api-key"),
  configPath: document.querySelector("#config-path"),
  input: document.querySelector("#input"),
  polish: document.querySelector("#polish"),
  output: document.querySelector("#output"),
  status: document.querySelector("#result-status"),
  version: document.querySelector("#version"),
};

async function loadConfig() {
  if (!invoke) {
    els.configPath.textContent = "Run the app with `cargo tauri dev` to load configuration.";
    return;
  }

  try {
    const config = await invoke("get_config");
    els.hotkey.textContent = config.hotkey;
    els.provider.textContent = `${config.model} · ${config.base_url}`;
    els.apiKey.textContent = config.api_key_present
      ? `set via ${config.api_key_env}`
      : `missing ${config.api_key_env}`;
    els.apiKey.classList.toggle("warn", !config.api_key_present);
    els.configPath.textContent = config.config_path;
    els.version.textContent = config.version;
  } catch (error) {
    setStatus(String(error), true);
  }
}

function setStatus(message, isError = false) {
  els.status.textContent = message;
  els.status.classList.toggle("error", isError);
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

els.polish.addEventListener("click", polish);
els.input.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    polish();
  }
});

listen?.("refiny-state", (event) => {
  const payload = event.payload ?? {};
  if (payload.state === "error") {
    setStatus(payload.message, true);
  }
});

loadConfig();
