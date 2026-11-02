// Konto-Seite: registrieren, anmelden, Spiel verknüpfen (?link=CODE), Profil, Bestenliste.
// Die Sitzung liegt im localStorage unter demselben Schlüssel, den das Browser-Spiel liest;
// wer sich hier anmeldet, ist im Browser-Spiel derselben Seite sofort mit angemeldet.

const TOKEN_KEY = "totenrunde.session";

const TEXTS = {
  de: {
    navPlay: "Im Browser spielen",
    navBoard: "Bestenliste",
    linkEyebrow: "Spiel verbinden",
    linkText: "Melde dich an oder erstelle ein Konto. Vergleiche danach den Code mit deinem eigenen Spiel und bestätige die Verbindung.",
    linkTextSignedIn: "Dieses Spiel erhält Zugriff auf dein Konto {name}. Bestätige nur, wenn du die Verbindung selbst in deinem eigenen Spiel gestartet hast und dort derselbe Code steht.",
    linkConfirm: "Dieses Spiel verbinden",
    linkConnecting: "Dein Spiel wird verbunden …",
    linkDoneEyebrow: "Verbunden",
    linkDone: "Dein Spiel ist jetzt angemeldet. Du kannst dieses Fenster schließen und zurück ins Spiel wechseln.",
    linkExpired: "Dieser Code ist abgelaufen. Klicke im Spiel erneut auf „Konto erstellen / Anmelden“.",
    tabRegister: "Konto erstellen",
    tabLogin: "Anmelden",
    name: "Spielername",
    nameHint: "3–16 Zeichen: Buchstaben, Ziffern, _ - .",
    password: "Passwort",
    passHint: "Mindestens 8 Zeichen.",
    password2: "Passwort wiederholen",
    register: "Konto erstellen",
    login: "Anmelden",
    profile: "Dein Konto",
    since: "Dabei seit {date}",
    play: "Jetzt spielen",
    logout: "Abmelden",
    deleteTitle: "Konto löschen",
    deleteText: "Löscht dein Konto mit allen Statistiken. Das lässt sich nicht rückgängig machen.",
    deleteButton: "Endgültig löschen",
    board: "Bestenliste",
    boardHint: "Höchste erreichte Runde in Koop-Partien, bei Gleichstand zählen die Kills.",
    boardEmpty: "Noch keine Partien – sei die erste Gruppe auf der Liste!",
    colName: "Name",
    colRound: "Runde",
    colKills: "Kills",
    colGames: "Partien",
    online: "{online} online · {lobbies} Lobbys",
    footer: "Für den Koop-Modus von Totenrunde. Gespeichert werden nur Name, Passwort-Hash und Spielstatistik.",
    statGames: "Partien",
    statBest: "Beste Runde",
    statKills: "Kills",
    statHeadshots: "Kopfschüsse",
    statRevives: "Wiederbelebt",
    statDowns: "Niedergestreckt",
    statPoints: "Punkte",
    errInvalidName: "Der Name muss 3–16 Zeichen lang sein (Buchstaben, Ziffern, _ - .).",
    errShortPass: "Das Passwort braucht mindestens 8 Zeichen.",
    errMismatch: "Die Passwörter stimmen nicht überein.",
    errTaken: "Dieser Name ist schon vergeben.",
    errWrong: "Name oder Passwort stimmt nicht.",
    errRate: "Zu viele Versuche. Bitte warte ein paar Minuten.",
    errNetwork: "Der Server ist gerade nicht erreichbar.",
    errGeneric: "Das hat nicht geklappt. Bitte versuche es erneut.",
  },
  en: {
    navPlay: "Play in the browser",
    navBoard: "Leaderboard",
    linkEyebrow: "Connect game",
    linkText: "Sign in or create an account. Then compare the code with your own game and confirm the connection.",
    linkTextSignedIn: "This game will receive access to your account {name}. Only confirm if you started the connection in your own game and it shows the same code.",
    linkConfirm: "Connect this game",
    linkConnecting: "Connecting your game …",
    linkDoneEyebrow: "Connected",
    linkDone: "Your game is signed in now. You can close this window and switch back to the game.",
    linkExpired: "This code has expired. Click “Create account / Sign in” in the game again.",
    tabRegister: "Create account",
    tabLogin: "Sign in",
    name: "Player name",
    nameHint: "3–16 characters: letters, digits, _ - .",
    password: "Password",
    passHint: "At least 8 characters.",
    password2: "Repeat password",
    register: "Create account",
    login: "Sign in",
    profile: "Your account",
    since: "Member since {date}",
    play: "Play now",
    logout: "Sign out",
    deleteTitle: "Delete account",
    deleteText: "Deletes your account with all statistics. This cannot be undone.",
    deleteButton: "Delete permanently",
    board: "Leaderboard",
    boardHint: "Highest round reached in co-op matches; kills break ties.",
    boardEmpty: "No matches yet – be the first squad on the list!",
    colName: "Name",
    colRound: "Round",
    colKills: "Kills",
    colGames: "Matches",
    online: "{online} online · {lobbies} lobbies",
    footer: "For the Totenrunde co-op mode. Only your name, a password hash and match statistics are stored.",
    statGames: "Matches",
    statBest: "Best round",
    statKills: "Kills",
    statHeadshots: "Headshots",
    statRevives: "Revives",
    statDowns: "Downs",
    statPoints: "Points",
    errInvalidName: "The name must be 3–16 characters (letters, digits, _ - .).",
    errShortPass: "The password needs at least 8 characters.",
    errMismatch: "The passwords do not match.",
    errTaken: "This name is already taken.",
    errWrong: "Wrong name or password.",
    errRate: "Too many attempts. Please wait a few minutes.",
    errNetwork: "The server cannot be reached right now.",
    errGeneric: "That did not work. Please try again.",
  },
};

// Sprache wie im Spiel: gespeicherte Einstellung, sonst Browsersprache (Deutsch oder Englisch)
function language() {
  try {
    const saved = JSON.parse(localStorage.getItem("totenrunde.settings") ?? "{}").language;
    if (saved && saved !== "auto") {
      return saved === "de" ? "de" : "en";
    }
  } catch {
    // ungültige Einstellungen ignorieren
  }
  const preferred = navigator.languages?.length ? navigator.languages : [navigator.language];
  return preferred.some((l) => l?.toLowerCase().startsWith("de")) ? "de" : "en";
}

const lang = language();
const texts = TEXTS[lang];
document.documentElement.lang = lang;

function t(key, values = {}) {
  let text = texts[key] ?? TEXTS.en[key] ?? key;
  for (const [name, value] of Object.entries(values)) {
    text = text.replaceAll(`{${name}}`, String(value));
  }
  return text;
}

for (const el of document.querySelectorAll("[data-t]")) {
  el.textContent = t(el.dataset.t);
}
document.title = `Totenrunde – ${t("profile")}`;

const $ = (id) => document.getElementById(id);

function token() {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

function setToken(value) {
  try {
    if (value) {
      localStorage.setItem(TOKEN_KEY, value);
    } else {
      localStorage.removeItem(TOKEN_KEY);
    }
  } catch {
    // ohne localStorage bleibt man nur bis zum Neuladen angemeldet
  }
}

async function api(path, { method = "GET", body, auth = false, authToken } = {}) {
  const headers = {};
  if (body !== undefined) {
    headers["Content-Type"] = "application/json";
  }
  const session = authToken ?? (auth ? token() : null);
  if (session) {
    headers.Authorization = `Bearer ${session}`;
  }
  let response;
  try {
    response = await fetch(path, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    return { ok: false, status: 0, data: { error: "network" } };
  }
  let data = null;
  if (response.status !== 204) {
    data = await response.json().catch(() => null);
  }
  return { ok: response.ok, status: response.status, data };
}

function errorText(result) {
  switch (result.data?.error) {
    case "invalid": return t("errInvalidName");
    case "name_taken": return t("errTaken");
    case "wrong_credentials": return t("errWrong");
    case "rate_limited": return t("errRate");
    case "network": return t("errNetwork");
    default: return t("errGeneric");
  }
}

// --- Verknüpfungscode aus der Adresse (vom Spiel geöffnet) ---------------------------------

const params = new URLSearchParams(location.search);
const requestedCode = (params.get("link") ?? "").toUpperCase().replaceAll("-", "");
const linkCode = /^[A-HJ-NP-Z2-9]{8}$/.test(requestedCode) ? requestedCode : "";
let linkComplete = false;
let linkPending = false;
let linkSession = null;

function showLinkCode() {
  if (!linkCode) {
    return;
  }
  $("link-code").textContent = `${linkCode.slice(0, 4)}-${linkCode.slice(4)}`;
  $("link-banner").hidden = false;
}

async function completeLink() {
  if (!linkCode || !token() || linkComplete || linkPending || $("confirm-link").hidden) {
    return;
  }
  // Bei einem Kontowechsel in einem anderen Tab zuerst die neue Freigabe anzeigen.
  if (!linkSession || token() !== linkSession) {
    await refresh();
    return;
  }
  linkPending = true;
  $("confirm-link").disabled = true;
  $("link-text").textContent = t("linkConnecting");
  try {
    const result = await api("/api/account/link", {
      method: "POST",
      body: { code: linkCode },
      authToken: linkSession,
    });
    if (result.ok) {
      linkComplete = true;
      $("link-banner").hidden = true;
      $("confirm-link").hidden = true;
      $("link-done").hidden = false;
      // Code nur einmal verwenden; Neuladen zeigt das normale Profil
      history.replaceState(null, "", location.pathname);
    } else if (result.status === 404) {
      $("link-text").textContent = t("linkExpired");
      $("confirm-link").hidden = true;
    } else {
      $("link-text").textContent = errorText(result);
    }
  } finally {
    linkPending = false;
    $("confirm-link").disabled = false;
  }
}

$("confirm-link").addEventListener("click", completeLink);

// --- Anmelden und Registrieren -------------------------------------------------------------

function selectTab(register) {
  $("tab-register").setAttribute("aria-selected", String(register));
  $("tab-login").setAttribute("aria-selected", String(!register));
  $("form-register").hidden = !register;
  $("form-login").hidden = register;
}

$("tab-register").addEventListener("click", () => selectTab(true));
$("tab-login").addEventListener("click", () => selectTab(false));

const NAME_PATTERN = /^[A-Za-z0-9_.-]{3,16}$/;

async function submit(form, button, errorEl, run) {
  errorEl.textContent = "";
  button.disabled = true;
  try {
    await run();
  } finally {
    button.disabled = false;
  }
}

$("form-register").addEventListener("submit", (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const name = $("reg-name").value.trim();
  const password = $("reg-pass").value;
  const errorEl = $("reg-error");
  if (!NAME_PATTERN.test(name)) {
    errorEl.textContent = t("errInvalidName");
    return;
  }
  if (password.length < 8) {
    errorEl.textContent = t("errShortPass");
    return;
  }
  if (password !== $("reg-pass2").value) {
    errorEl.textContent = t("errMismatch");
    return;
  }
  submit(form, form.querySelector("button"), errorEl, async () => {
    const result = await api("/api/account/register", { method: "POST", body: { name, password } });
    if (!result.ok) {
      errorEl.textContent = errorText(result);
      return;
    }
    setToken(result.data.token);
    form.reset();
    await refresh();
  });
});

$("form-login").addEventListener("submit", (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const name = $("login-name").value.trim();
  const password = $("login-pass").value;
  const errorEl = $("login-error");
  submit(form, form.querySelector("button"), errorEl, async () => {
    const result = await api("/api/account/login", { method: "POST", body: { name, password } });
    if (!result.ok) {
      errorEl.textContent = errorText(result);
      return;
    }
    setToken(result.data.token);
    form.reset();
    await refresh();
  });
});

$("logout").addEventListener("click", async () => {
  await api("/api/account/logout", { method: "POST", auth: true });
  setToken(null);
  await refresh();
});

$("form-delete").addEventListener("submit", (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const errorEl = $("del-error");
  submit(form, form.querySelector("button"), errorEl, async () => {
    const result = await api("/api/account", {
      method: "DELETE",
      body: { password: $("del-pass").value },
      auth: true,
    });
    if (!result.ok) {
      errorEl.textContent = errorText(result);
      return;
    }
    setToken(null);
    form.reset();
    await refresh();
    await loadBoard();
  });
});

// --- Profil --------------------------------------------------------------------------------

function renderStats(stats) {
  const list = $("me-stats");
  list.replaceChildren();
  const rows = [
    ["statGames", stats.games],
    ["statBest", stats.best_round],
    ["statKills", stats.kills],
    ["statHeadshots", stats.headshots],
    ["statRevives", stats.revives],
    ["statDowns", stats.downs],
    ["statPoints", stats.points],
  ];
  for (const [key, value] of rows) {
    const item = document.createElement("div");
    const dt = document.createElement("dt");
    dt.textContent = t(key);
    const dd = document.createElement("dd");
    dd.textContent = Number(value ?? 0).toLocaleString(lang);
    item.append(dt, dd);
    list.append(item);
  }
}

async function refresh() {
  let me = null;
  const session = token();
  if (session) {
    const result = await api("/api/account", { authToken: session });
    if (result.ok) {
      me = result.data;
    } else if (result.status === 401) {
      setToken(null);
    }
  }
  linkSession = me ? session : null;
  $("signed-in").hidden = !me;
  $("signed-out").hidden = Boolean(me);
  $("confirm-link").hidden = !me || !linkCode || linkComplete;
  if (linkCode && !linkComplete) {
    $("link-text").textContent = me ? t("linkTextSignedIn", { name: me.name }) : t("linkText");
  }
  if (me) {
    $("me-name").textContent = me.name;
    const since = new Date(me.created * 1000).toLocaleDateString(lang, { dateStyle: "long" });
    $("me-since").textContent = t("since", { date: since });
    renderStats(me.stats ?? {});
  }
}

// --- Bestenliste ---------------------------------------------------------------------------

async function loadBoard() {
  const [board, status] = await Promise.all([api("/api/leaderboard"), api("/api/mp/status")]);
  const body = $("board");
  body.replaceChildren();
  const entries = board.ok && Array.isArray(board.data) ? board.data : [];
  entries.forEach((entry, index) => {
    const row = document.createElement("tr");
    for (const value of [index + 1, entry.name, entry.best_round, entry.kills, entry.games]) {
      const cell = document.createElement("td");
      cell.textContent = typeof value === "number" ? value.toLocaleString(lang) : value;
      row.append(cell);
    }
    body.append(row);
  });
  $("board-empty").hidden = entries.length > 0;
  if (status.ok && status.data) {
    $("online").textContent = t("online", status.data);
  }
}

showLinkCode();
if (linkCode && !token()) {
  // Neue Spieler kommen meist ohne Konto aus dem Spiel
  selectTab(true);
}
await refresh();
await loadBoard();
