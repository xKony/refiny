const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke;
const listen = tauri?.event?.listen;

const pill = document.querySelector("#pill");
const label = document.querySelector("#label");
let hideTimer;

function setState(state, message) {
  pill.dataset.state = state;
  label.textContent = message;

  clearTimeout(hideTimer);
  if (state !== "working" && invoke) {
    hideTimer = setTimeout(() => invoke("hide_overlay"), 1800);
  }
}

listen?.("refiny-state", (event) => {
  const payload = event.payload ?? {};
  setState(payload.state ?? "error", payload.message ?? "");
});

setState("working", "Polishing…");
