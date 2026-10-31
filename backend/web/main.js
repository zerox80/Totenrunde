const loading = document.getElementById("loading");
const error = document.getElementById("error");
const errorMessage = document.getElementById("error-message");
const canvas = document.getElementById("game-canvas");

// Ohne Hardwarebeschleunigung rendert der Browser auf der CPU; das Spiel ist dann unspielbar
// langsam, auch auf schnellen Rechnern. Darauf deutlich hinweisen.
function warnIfSoftwareRendering() {
  const gl = document.createElement("canvas").getContext("webgl2");
  const info = gl && gl.getExtension("WEBGL_debug_renderer_info");
  const renderer = info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : "";
  gl?.getExtension("WEBGL_lose_context")?.loseContext();
  console.info("WebGL2-Renderer:", renderer || "unbekannt");
  if (!/swiftshader|llvmpipe|softpipe|software|basic render/i.test(renderer)) {
    return;
  }
  const hint = document.createElement("div");
  hint.setAttribute("role", "alert");
  hint.style.cssText = "position:fixed;left:50%;top:1rem;transform:translateX(-50%);z-index:10;" +
    "max-width:42rem;padding:.9rem 1.2rem;border-left:4px solid #9e0d0a;background:#14100f;" +
    "color:#f4f4f5;font:15px/1.5 system-ui,sans-serif;box-shadow:0 4px 24px #000a";
  hint.textContent = "Die Grafikbeschleunigung deines Browsers ist ausgeschaltet (" + renderer +
    "). Das Spiel läuft dadurch extrem langsam. Einschalten: Browser-Einstellungen → System → " +
    "„Grafikbeschleunigung verwenden, wenn verfügbar“, danach den Browser neu starten.";
  document.body.append(hint);
}

// WebGPU, wenn der Browser einen echten Grafikadapter dafür liefert, sonst WebGL2.
// Zum Testen erzwingen: ?renderer=webgl2 bzw. ?renderer=webgpu
async function chooseRenderer() {
  const forced = new URLSearchParams(location.search).get("renderer");
  if (forced === "webgl2" || forced === "webgpu") {
    return forced;
  }
  if (!navigator.gpu) {
    return "webgl2";
  }
  try {
    const adapter = await navigator.gpu.requestAdapter({ powerPreference: "high-performance" });
    const software = adapter?.info?.isFallbackAdapter ?? adapter?.isFallbackAdapter;
    return adapter && !software ? "webgpu" : "webgl2";
  } catch {
    return "webgl2";
  }
}

// Scheitert WebGPU trotzdem beim Start (Absturz im Spielcode), mit WebGL2 neu laden.
// Nur Fehler aus dem Spiel zählen, nicht die von Browser-Erweiterungen.
function fallBackToWebGl2OnCrash() {
  const crashed = (event) => {
    const fromGame = event.error instanceof WebAssembly.RuntimeError
      || (event.filename ?? "").includes("/pkg/");
    if (fromGame) {
      const url = new URL(location.href);
      url.searchParams.set("renderer", "webgl2");
      location.replace(url);
    }
  };
  window.addEventListener("error", crashed);
  setTimeout(() => window.removeEventListener("error", crashed), 30000);
}

warnIfSoftwareRendering();

try {
  const renderer = await chooseRenderer();
  console.info("Grafikschnittstelle:", renderer);
  if (renderer === "webgpu" && !new URLSearchParams(location.search).has("renderer")) {
    fallBackToWebGl2OnCrash();
  }
  const name = renderer === "webgpu" ? "totenrunde_webgpu" : "totenrunde";
  const { default: init } = await import(`./pkg/${name}.js`);
  await init({ module_or_path: `./pkg/${name}_bg.wasm` });
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
