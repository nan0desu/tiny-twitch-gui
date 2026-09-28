# twitchclient-live

Десктопный клиент для просмотра live-стримов на которые ты подписана. Tauri + Rust + Vanilla JS.

## Возможности

- Авторизация через Twitch OAuth (Implicit Flow, без client secret)
- Список текущих стримов с превьюхами, зрителями и тайтлом
- Автообновление каждые 60 секунд
- Сессия сохраняется между перезапусками (~60 дней)
- Настраиваемая команда для открытия стрима (mpv, vlc, браузер и т.д.)

## Требования

- [Rust](https://rustup.rs/) (или через `mise install rust`)
- Node.js 24 LTS (или через `mise install node`)
- Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libssl-dev`

## Настройка Twitch-приложения

1. Зайди на [dev.twitch.tv/console](https://dev.twitch.tv/console) → **Register Your Application**
2. OAuth Redirect URL: `http://localhost:17563/callback`
3. Category: Application Integration
4. Скопируй **Client ID**

## Разработка
```bash
fedora:
dnf install gcc gcc-c++ gtk3-devel libsoup3-devel gdk-pixbuf2-devel pango-devel javascriptcoregtk4.1-devel atk-devel glib2-devel cairo-gobject-devel webkit2gtk4.1-devel
```

```bash
npm install
TWITCH_CLIENT_ID=xxxxxx npm run tauri dev
```

## Сборка

**Просто бинарь** (без пакета):
```bash
TWITCH_CLIENT_ID=xxxxxx npm run tauri build -- --no-bundle
```
Результат: `src-tauri/target/release/twitchclient-live`

**С пакетом** (требует `xdg-utils` на машине сборки для AppImage):
```bash
TWITCH_CLIENT_ID=xxxxxx npm run tauri build
```

Артефакты в `src-tauri/target/release/bundle/`:

| Формат | Путь |
|--------|------|
| .deb | `deb/twitchclient-live_*.deb` |
| .rpm | `rpm/twitchclient-live-*.rpm` |
| AppImage | требует `sudo dnf install xdg-utils` |

> Client ID запекается в бинарник во время сборки — в рантайме переменная окружения не нужна.

## Конфиг

Создаётся автоматически при первом запуске: `~/.config/twitchclient-live/config.toml`

```toml
[thumbnails]
# Размер, который запрашивается у Twitch, в пикселях. Лучше держать не меньше
# самой широкой карточки, которую даёт ползунок зума (520 px), иначе превью мылит.
width = 640
height = 360
# Как часто перезапрашивается превью, в минутах. Отсчёт идёт от старта самого
# стрима, поэтому карточки обновляются вразнобой, а не все разом.
# Стрим моложе этого значения остаётся с первым кадром. 0 — не обновлять.
refresh_minutes = 10

[player]
# Плейсхолдеры: {channel}, {url}
command = ["xdg-open", "https://twitch.tv/{channel}"]
```

Примеры команд:

```toml
# mpv через Streamlink
command = ["streamlink", "twitch.tv/{channel}", "best"]

# VLC
command = ["vlc", "https://twitch.tv/{channel}"]

# Firefox
command = ["firefox", "https://twitch.tv/{channel}"]
```
