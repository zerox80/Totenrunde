import init from "./pkg/totenrunde.js";

const loading = document.getElementById("loading");
const error = document.getElementById("error");
const errorMessage = document.getElementById("error-message");
const canvas = document.getElementById("game-canvas");

try {
  await init({ module_or_path: "./pkg/totenrunde_bg.wasm" });
  loading.hidden = true;
  canvas.focus();
} catch (cause) {
  console.error("Totenrunde konnte nicht gestartet werden:", cause);
  loading.hidden = true;
  error.hidden = false;
  errorMessage.textContent = cause instanceof Error
    ? `Fehler beim Laden des Spiels: ${cause.message}`
    : "Das Spiel konnte nicht geladen werden.";
}
