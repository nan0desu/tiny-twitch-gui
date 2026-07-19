const { invoke } = window.__TAURI__.core;

const REFRESH_INTERVAL_MS = 60_000;
let appConfig = null;

const authScreen = document.getElementById("auth-screen");
const streamsScreen = document.getElementById("streams-screen");
const loginBtn = document.getElementById("login-btn");
const authStatus = document.getElementById("auth-status");
const streamsGrid = document.getElementById("streams-grid");
const emptyState = document.getElementById("empty-state");
const streamCount = document.getElementById("stream-count");
const lastUpdated = document.getElementById("last-updated");
const refreshBtn = document.getElementById("refresh-btn");
const logoutBtn = document.getElementById("logout-btn");
const zoomSlider = document.getElementById("zoom-slider");

let refreshTimer = null;
const cardMap = new Map();

function formatViewers(n) {
  return n >= 1000 ? (n / 1000).toFixed(1) + "K" : String(n);
}

function timeSince(date) {
  const s = Math.floor((Date.now() - date) / 1000);
  if (s < 60) return "now";
  return `${Math.floor(s / 60)} мин назад`;
}

function renderStream(stream) {
  const card = document.createElement("div");
  card.className = "stream-card";
  card.innerHTML = `
    <div class="thumbnail-wrap">
      <img src="${stream.thumbnail_url}?t=${Date.now()}" alt="${stream.user_name}" loading="lazy" />
      <span class="live-badge">LIVE</span>
      ${stream.tags.some((t) => t.toLowerCase() === "2k") ? `<span class="tag-badge">2K</span>` : ""}
      <span class="viewers-badge">${formatViewers(stream.viewer_count)}</span>
    </div>
    <div class="card-info">
      <div class="card-user">${stream.user_name}</div>
      <div class="card-title" title="${stream.title}">${stream.title || "—"}</div>
      ${stream.game_name ? `<div class="card-game">${stream.game_name}</div>` : ""}
    </div>
  `;
  card.addEventListener("click", () => {
    invoke("open_stream", { channel: stream.user_login }).catch((e) =>
      console.error(e),
    );
  });
  return card;
}

function updateCard(card, stream) {
  card.querySelector(".viewers-badge").textContent = formatViewers(
    stream.viewer_count,
  );
  const titleEl = card.querySelector(".card-title");
  titleEl.textContent = stream.title || "—";
  titleEl.title = stream.title;
  const gameEl = card.querySelector(".card-game");
  if (gameEl) gameEl.textContent = stream.game_name;
}

async function loadStreams() {
  refreshBtn.classList.add("spinning");
  try {
    const streams = await invoke("get_streams");
    if (streams.length === 0) {
      emptyState.classList.remove("hidden");
      cardMap.forEach((card) => card.remove());
      cardMap.clear();
    } else {
      emptyState.classList.add("hidden");
      const seen = new Set();
      streams.forEach((s) => {
        seen.add(s.user_login);
        if (cardMap.has(s.user_login)) {
          updateCard(cardMap.get(s.user_login), s);
        } else {
          const card = renderStream(s);
          cardMap.set(s.user_login, card);
        }
        streamsGrid.appendChild(cardMap.get(s.user_login));
      });
      cardMap.forEach((card, login) => {
        if (!seen.has(login)) {
          card.remove();
          cardMap.delete(login);
        }
      });
    }
    streamCount.textContent = streams.length;
    lastUpdated.textContent = timeSince(Date.now());
  } catch (e) {
    if (e === "not_authenticated") showAuth();
  } finally {
    refreshBtn.classList.remove("spinning");
  }
}

function showAuth() {
  clearInterval(refreshTimer);
  cardMap.clear();
  streamsGrid.innerHTML = "";
  authScreen.classList.remove("hidden");
  streamsScreen.classList.add("hidden");
}

function showStreams() {
  authScreen.classList.add("hidden");
  streamsScreen.classList.remove("hidden");
  loadStreams();
  refreshTimer = setInterval(loadStreams, REFRESH_INTERVAL_MS);
}

loginBtn.addEventListener("click", async () => {
  loginBtn.disabled = true;
  authStatus.textContent = "Открываю браузер...";
  try {
    await invoke("start_auth");
    authStatus.textContent = "Жду авторизацию в браузере...";
    await invoke("finish_auth");
    showStreams();
  } catch (e) {
    authStatus.textContent = `Ошибка: ${e}`;
    loginBtn.disabled = false;
  }
});

refreshBtn.addEventListener("click", loadStreams);

const ZOOM_KEY = "twitch-live-zoom";
const savedZoom = localStorage.getItem(ZOOM_KEY) || "280";
zoomSlider.value = savedZoom;
document.documentElement.style.setProperty(
  "--card-min-width",
  savedZoom + "px",
);

zoomSlider.addEventListener("input", () => {
  const v = zoomSlider.value;
  document.documentElement.style.setProperty("--card-min-width", v + "px");
  localStorage.setItem(ZOOM_KEY, v);
});

logoutBtn.addEventListener("click", async () => {
  await invoke("logout");
  showAuth();
});

// При запуске загружаем конфиг и проверяем авторизацию
(async () => {
  appConfig = await invoke("get_config");
  const authed = await invoke("is_authenticated");
  if (authed) showStreams();
  else showAuth();
})();
