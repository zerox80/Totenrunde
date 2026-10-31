// Texte der Ladeseite. Die Sprache folgt derselben Wahl wie im Spiel (src/base/i18n.rs):
// gespeicherte Einstellung, sonst Browsersprache, sonst Englisch.

const TEXTS = {
  de: {
    loading: "Das Spiel wird geladen …",
    loadingHint: "Bitte verwende einen aktuellen Browser mit WebAssembly und WebGPU oder WebGL 2.",
    errorTitle: "Start nicht möglich",
    errorHint: "Prüfe deine Internetverbindung und lade die Seite neu. Totenrunde benötigt einen aktuellen Browser mit WebAssembly und WebGPU oder WebGL 2.",
    loadError: "Fehler beim Laden des Spiels: ",
    loadFailed: "Das Spiel konnte nicht geladen werden.",
    software: "Die Grafikbeschleunigung deines Browsers ist ausgeschaltet ({renderer}). Das Spiel läuft dadurch extrem langsam. Einschalten: Browser-Einstellungen → System → „Grafikbeschleunigung verwenden, wenn verfügbar“, danach den Browser neu starten.",
    canvas: "Totenrunde-Spielansicht",
  },
  en: {
    loading: "Loading the game …",
    loadingHint: "Please use an up-to-date browser with WebAssembly and WebGPU or WebGL 2.",
    errorTitle: "Unable to start",
    errorHint: "Check your internet connection and reload the page. Totenrunde needs an up-to-date browser with WebAssembly and WebGPU or WebGL 2.",
    loadError: "Error while loading the game: ",
    loadFailed: "The game could not be loaded.",
    software: "Your browser's graphics acceleration is turned off ({renderer}). The game will run extremely slowly. To turn it on: browser settings → System → “Use graphics acceleration when available”, then restart the browser.",
    canvas: "Totenrunde game view",
  },
  fr: {
    loading: "Chargement du jeu …",
    loadingHint: "Utilise un navigateur récent compatible WebAssembly et WebGPU ou WebGL 2.",
    errorTitle: "Impossible de démarrer",
    errorHint: "Vérifie ta connexion Internet et recharge la page. Totenrunde nécessite un navigateur récent avec WebAssembly et WebGPU ou WebGL 2.",
    loadError: "Erreur lors du chargement du jeu : ",
    loadFailed: "Le jeu n'a pas pu être chargé.",
    software: "L'accélération graphique de ton navigateur est désactivée ({renderer}). Le jeu sera donc extrêmement lent. Pour l'activer : paramètres du navigateur → Système → « Utiliser l'accélération graphique si disponible », puis redémarre le navigateur.",
    canvas: "Vue de jeu Totenrunde",
  },
  es: {
    loading: "Cargando el juego …",
    loadingHint: "Usa un navegador actualizado con WebAssembly y WebGPU o WebGL 2.",
    errorTitle: "No se puede iniciar",
    errorHint: "Comprueba tu conexión a Internet y recarga la página. Totenrunde necesita un navegador actualizado con WebAssembly y WebGPU o WebGL 2.",
    loadError: "Error al cargar el juego: ",
    loadFailed: "No se pudo cargar el juego.",
    software: "La aceleración gráfica de tu navegador está desactivada ({renderer}). Por eso el juego irá extremadamente lento. Para activarla: ajustes del navegador → Sistema → «Usar aceleración gráfica cuando esté disponible» y reinicia el navegador.",
    canvas: "Vista del juego Totenrunde",
  },
  it: {
    loading: "Caricamento del gioco …",
    loadingHint: "Usa un browser aggiornato con WebAssembly e WebGPU o WebGL 2.",
    errorTitle: "Avvio non riuscito",
    errorHint: "Controlla la connessione a Internet e ricarica la pagina. Totenrunde richiede un browser aggiornato con WebAssembly e WebGPU o WebGL 2.",
    loadError: "Errore durante il caricamento del gioco: ",
    loadFailed: "Impossibile caricare il gioco.",
    software: "L'accelerazione grafica del browser è disattivata ({renderer}). Il gioco sarà quindi estremamente lento. Per attivarla: impostazioni del browser → Sistema → «Usa l'accelerazione grafica se disponibile», poi riavvia il browser.",
    canvas: "Vista di gioco di Totenrunde",
  },
  pt: {
    loading: "Carregando o jogo …",
    loadingHint: "Use um navegador atualizado com WebAssembly e WebGPU ou WebGL 2.",
    errorTitle: "Não foi possível iniciar",
    errorHint: "Verifique sua conexão com a internet e recarregue a página. O Totenrunde precisa de um navegador atualizado com WebAssembly e WebGPU ou WebGL 2.",
    loadError: "Erro ao carregar o jogo: ",
    loadFailed: "Não foi possível carregar o jogo.",
    software: "A aceleração gráfica do seu navegador está desativada ({renderer}). Por isso o jogo vai rodar extremamente lento. Para ativar: configurações do navegador → Sistema → “Usar aceleração gráfica quando disponível” e reinicie o navegador.",
    canvas: "Tela do jogo Totenrunde",
  },
  nl: {
    loading: "Het spel wordt geladen …",
    loadingHint: "Gebruik een actuele browser met WebAssembly en WebGPU of WebGL 2.",
    errorTitle: "Starten niet mogelijk",
    errorHint: "Controleer je internetverbinding en laad de pagina opnieuw. Totenrunde heeft een actuele browser met WebAssembly en WebGPU of WebGL 2 nodig.",
    loadError: "Fout bij het laden van het spel: ",
    loadFailed: "Het spel kon niet worden geladen.",
    software: "De grafische versnelling van je browser staat uit ({renderer}). Het spel loopt daardoor extreem traag. Inschakelen: browserinstellingen → Systeem → ‘Grafische versnelling gebruiken indien beschikbaar’ en start de browser daarna opnieuw.",
    canvas: "Spelweergave van Totenrunde",
  },
  pl: {
    loading: "Wczytywanie gry …",
    loadingHint: "Użyj aktualnej przeglądarki z obsługą WebAssembly oraz WebGPU lub WebGL 2.",
    errorTitle: "Nie można uruchomić",
    errorHint: "Sprawdź połączenie z internetem i odśwież stronę. Totenrunde wymaga aktualnej przeglądarki z WebAssembly oraz WebGPU lub WebGL 2.",
    loadError: "Błąd podczas wczytywania gry: ",
    loadFailed: "Nie udało się wczytać gry.",
    software: "Akceleracja graficzna w przeglądarce jest wyłączona ({renderer}). Gra będzie przez to działać bardzo wolno. Włącz ją: ustawienia przeglądarki → System → „Używaj akceleracji graficznej, gdy jest dostępna”, a potem uruchom przeglądarkę ponownie.",
    canvas: "Widok gry Totenrunde",
  },
  cs: {
    loading: "Načítání hry …",
    loadingHint: "Použij aktuální prohlížeč s podporou WebAssembly a WebGPU nebo WebGL 2.",
    errorTitle: "Hru nelze spustit",
    errorHint: "Zkontroluj připojení k internetu a obnov stránku. Totenrunde vyžaduje aktuální prohlížeč s WebAssembly a WebGPU nebo WebGL 2.",
    loadError: "Chyba při načítání hry: ",
    loadFailed: "Hru se nepodařilo načíst.",
    software: "Grafická akcelerace v prohlížeči je vypnutá ({renderer}). Hra proto poběží extrémně pomalu. Zapnutí: nastavení prohlížeče → Systém → „Použít grafickou akceleraci, je-li k dispozici“, poté prohlížeč restartuj.",
    canvas: "Herní zobrazení Totenrunde",
  },
  tr: {
    loading: "Oyun yükleniyor …",
    loadingHint: "Lütfen WebAssembly ve WebGPU ya da WebGL 2 destekleyen güncel bir tarayıcı kullan.",
    errorTitle: "Başlatılamıyor",
    errorHint: "İnternet bağlantını kontrol et ve sayfayı yenile. Totenrunde, WebAssembly ve WebGPU ya da WebGL 2 destekleyen güncel bir tarayıcı gerektirir.",
    loadError: "Oyun yüklenirken hata: ",
    loadFailed: "Oyun yüklenemedi.",
    software: "Tarayıcının grafik hızlandırması kapalı ({renderer}). Oyun bu yüzden çok yavaş çalışır. Açmak için: tarayıcı ayarları → Sistem → “Kullanılabilir olduğunda grafik hızlandırmayı kullan”, ardından tarayıcıyı yeniden başlat.",
    canvas: "Totenrunde oyun görünümü",
  },
  ru: {
    loading: "Игра загружается …",
    loadingHint: "Используй современный браузер с поддержкой WebAssembly и WebGPU или WebGL 2.",
    errorTitle: "Не удалось запустить",
    errorHint: "Проверь подключение к интернету и перезагрузи страницу. Для Totenrunde нужен современный браузер с WebAssembly и WebGPU или WebGL 2.",
    loadError: "Ошибка при загрузке игры: ",
    loadFailed: "Не удалось загрузить игру.",
    software: "Аппаратное ускорение графики в браузере выключено ({renderer}). Поэтому игра будет работать крайне медленно. Включить: настройки браузера → Система → «Использовать аппаратное ускорение (при наличии)», затем перезапусти браузер.",
    canvas: "Игровой экран Totenrunde",
  },
  uk: {
    loading: "Гра завантажується …",
    loadingHint: "Використовуй сучасний браузер із підтримкою WebAssembly та WebGPU або WebGL 2.",
    errorTitle: "Не вдалося запустити",
    errorHint: "Перевір підключення до інтернету й перезавантаж сторінку. Для Totenrunde потрібен сучасний браузер із WebAssembly та WebGPU або WebGL 2.",
    loadError: "Помилка під час завантаження гри: ",
    loadFailed: "Не вдалося завантажити гру.",
    software: "Апаратне прискорення графіки в браузері вимкнено ({renderer}). Тому гра працюватиме дуже повільно. Увімкнути: налаштування браузера → Система → «Використовувати апаратне прискорення (якщо доступно)», потім перезапусти браузер.",
    canvas: "Ігровий екран Totenrunde",
  },
};

function chooseLanguage() {
  try {
    const saved = JSON.parse(localStorage.getItem("totenrunde.settings") ?? "{}").language;
    if (saved && saved in TEXTS) {
      return saved;
    }
  } catch {
    // Kein Zugriff auf localStorage oder ungültige Daten: Browsersprache verwenden
  }
  for (const locale of navigator.languages ?? [navigator.language]) {
    const primary = String(locale).split(/[-_]/)[0].toLowerCase();
    if (primary in TEXTS) {
      return primary;
    }
  }
  return "en";
}

export const lang = chooseLanguage();

export function t(key, args = {}) {
  let text = TEXTS[lang][key] ?? TEXTS.en[key] ?? key;
  for (const [name, value] of Object.entries(args)) {
    text = text.replaceAll(`{${name}}`, value);
  }
  return text;
}

// Statische Texte der Seite übersetzen (Attribute data-i18n und data-i18n-label)
export function translatePage() {
  document.documentElement.lang = lang;
  for (const element of document.querySelectorAll("[data-i18n]")) {
    element.textContent = t(element.dataset.i18n);
  }
  for (const element of document.querySelectorAll("[data-i18n-label]")) {
    element.setAttribute("aria-label", t(element.dataset.i18nLabel));
  }
}
