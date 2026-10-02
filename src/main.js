const { invoke } = window.__TAURI__.core;
const { isPermissionGranted, requestPermission, sendNotification } =
  window.__TAURI__.notification;

const REFRESH_INTERVAL_MS = 60_000;
let appConfig = null;
let notifyLogins = new Set();
let notifyAllowed = false;

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
const aboutBtn = document.getElementById("about-btn");
const aboutDialog = document.getElementById("about-dialog");
const aboutVersion = document.getElementById("about-version");
const aboutConfig = document.getElementById("about-config");
const aboutRepo = document.getElementById("about-repo");
const aboutClose = document.getElementById("about-close");

let refreshTimer = null;
const cardMap = new Map();

// login -> started_at of the last broadcast seen. Keyed on started_at rather
// than a plain "was online" flag: a channel that drops out of one poll and
// returns within the same broadcast is not announced twice, while a genuinely
// new broadcast arrives with a new started_at and does notify.
const liveSince = new Map();
// The first load only fills liveSince in. Otherwise every launch would fire a
// burst of notifications about everyone who is already live.
let liveSeeded = false;

function notifyWentLive(stream) {
  const parts = [stream.title, stream.game_name].filter(Boolean);
  sendNotification({
    title: `${stream.user_name} is live`,
    body: parts.join(" · "),
  });
}

function checkWentLive(streams) {
  for (const s of streams) {
    const known = liveSince.get(s.user_login);
    const isNew = known === undefined || known !== s.started_at;
    if (
      isNew &&
      liveSeeded &&
      notifyAllowed &&
      (notifyLogins.has(s.user_login.toLowerCase()) ||
        notifyLogins.has(s.user_name.toLowerCase()))
    ) {
      notifyWentLive(s);
    }
  }

  liveSince.clear();
  streams.forEach((s) => liveSince.set(s.user_login, s.started_at));
  liveSeeded = true;
}

function formatViewers(n) {
  return n >= 1000 ? (n / 1000).toFixed(1) + "K" : String(n);
}

function timeSince(date) {
  const s = Math.floor((Date.now() - date) / 1000);
  if (s < 60) return "now";
  return `${Math.floor(s / 60)} min ago`;
}

// A preview is re-fetched once every refresh_minutes, counted from that
// stream's own start time. Streams started at different moments, so the grid
// never reloads as a whole. Below refresh_minutes the image is left alone.
function thumbUrl(stream) {
  const periodMs = (appConfig?.thumbnails?.refresh_minutes ?? 0) * 60_000;
  const ageMs = Date.now() - Date.parse(stream.started_at);
  if (periodMs <= 0 || !Number.isFinite(ageMs) || ageMs < periodMs) {
    return `${stream.thumbnail_url}?t=0`;
  }
  return `${stream.thumbnail_url}?t=${Math.floor(ageMs / periodMs)}`;
}

// Load the new frame aside and swap it in only once it is ready, otherwise
// <img> blinks empty for the duration of the request.
function refreshThumb(card, stream) {
  const img = card.querySelector(".thumbnail-wrap img");
  const next = thumbUrl(stream);
  if (img.dataset.thumb === next || img.dataset.pending === next) return;

  const pre = new Image();
  img.dataset.pending = next;
  pre.onload = () => {
    img.src = next;
    img.dataset.thumb = next;
    delete img.dataset.pending;
  };
  pre.onerror = () => {
    // Keep the old frame and retry on the next poll.
    delete img.dataset.pending;
  };
  pre.src = next;
}

function renderStream(stream) {
  const card = document.createElement("div");
  card.className = "stream-card";
  const thumb = thumbUrl(stream);
  card.innerHTML = `
    <div class="thumbnail-wrap">
      <img src="${thumb}" data-thumb="${thumb}" alt="${stream.user_name}" loading="lazy" />
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
  refreshThumb(card, stream);
}

async function loadStreams() {
  refreshBtn.classList.add("spinning");
  try {
    const streams = await invoke("get_streams");
    checkWentLive(streams);
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
  liveSince.clear();
  liveSeeded = false;
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
  authStatus.textContent = "Opening the browser...";
  try {
    await invoke("start_auth");
    authStatus.textContent = "Waiting for authorization in the browser...";
    await invoke("finish_auth");
    showStreams();
  } catch (e) {
    authStatus.textContent = `Error: ${e}`;
    loginBtn.disabled = false;
  }
});

refreshBtn.addEventListener("click", loadStreams);

const ZOOM_KEY = "twitchclient-live-zoom";
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

aboutBtn.addEventListener("click", async () => {
  try {
    const info = await invoke("app_info");
    aboutVersion.textContent = info.version;
    aboutConfig.textContent = info.config_path;
  } catch (e) {
    console.error(e);
  }
  aboutDialog.showModal();
});

aboutClose.addEventListener("click", () => aboutDialog.close());

aboutRepo.addEventListener("click", () =>
  invoke("open_repo").catch((e) => console.error(e)),
);

// Close on a backdrop click. Comparing against the bounding box rather than
// e.target, because clicks on the dialog's own padding also target the dialog.
aboutDialog.addEventListener("click", (e) => {
  const r = aboutDialog.getBoundingClientRect();
  const outside =
    e.clientX < r.left ||
    e.clientX > r.right ||
    e.clientY < r.top ||
    e.clientY > r.bottom;
  if (outside) aboutDialog.close();
});

// On startup load the config and check authorization
(async () => {
  appConfig = await invoke("get_config");
  notifyLogins = new Set(
    (appConfig.notify?.streamers ?? [])
      .map((s) => s.trim().toLowerCase())
      .filter(Boolean),
  );

  // Ask for permission before the first poll so notifyAllowed is settled by
  // then. With an empty list the user is never prompted at all.
  if (notifyLogins.size > 0) {
    notifyAllowed = await isPermissionGranted();
    if (!notifyAllowed) {
      notifyAllowed = (await requestPermission()) === "granted";
    }
  }

  const authed = await invoke("is_authenticated");
  if (authed) showStreams();
  else showAuth();
})();
