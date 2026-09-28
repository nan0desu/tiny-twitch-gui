# twitchclient-live

A desktop client for watching the live streams you follow. Tauri + Rust + Vanilla JS. 

## Features

- Twitch OAuth authorization (Implicit Flow, no client secret)
- List of live streams with previews, viewer counts, and titles
- Auto-refresh every 60 seconds
- Session persistence across restarts (~60 days)
- Customizable command to open streams (mpv, vlc, browser, etc.)

## Requirements

- [Rust](https://rustup.rs/) (or via `mise install rust`)
- Node.js 24 LTS (or via `mise install node`)
- Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libssl-dev`

## Twitch App Setup

1. Go to [dev.twitch.tv/console](https://dev.twitch.tv/console) → **Register Your Application**
2. OAuth Redirect URL: `http://localhost:17563/callback`
3. Category: Application Integration
4. Copy the **Client ID**

## Development
```bash
fedora:
dnf install gcc gcc-c++ gtk3-devel libsoup3-devel gdk-pixbuf2-devel pango-devel javascriptcoregtk4.1-devel atk-devel glib2-devel cairo-gobject-devel webkit2gtk4.1-devel
```

```bash
npm install
TWITCH_CLIENT_ID=xxxxxx npm run tauri dev
```

## Build

**Binary only** (no package):
```bash
TWITCH_CLIENT_ID=xxxxxx npm run tauri build -- --no-bundle
```
Result: `src-tauri/target/release/twitchclient-live`

**With package** (requires `xdg-utils` on the build machine for AppImage):
```bash
TWITCH_CLIENT_ID=xxxxxx npm run tauri build
```

Artifacts in `src-tauri/target/release/bundle/`:

| Format | Path |
|--------|------|
| .deb | `deb/twitchclient-live_*.deb` |
| .rpm | `rpm/twitchclient-live-*.rpm` |
| AppImage | requires `sudo dnf install xdg-utils` |

> The Client ID is baked into the binary during the build process—the environment variable is not needed at runtime.

## Configuration

Created automatically upon first launch: `~/.config/twitchclient-live/config.toml`

```toml
[thumbnails]
# Size requested from Twitch, in pixels. Worth keeping at or above the widest
# card the zoom slider can produce (520 px), otherwise previews look blurry.
width = 640
height = 360
# How often a preview is re-fetched, in minutes. Counted from that stream's own
# start time, so cards reload at different moments instead of all at once.
# A stream younger than this keeps its first frame. 0 disables refreshing.
refresh_minutes = 10

[player]
# Placeholders: {channel}, {url}
command = ["xdg-open", "https://twitch.tv/{channel}"]
```

Command examples:

```toml
# mpv via Streamlink
command = ["streamlink", "twitch.tv/{channel}", "best"]

# VLC
command = ["vlc", "https://twitch.tv/{channel}"]

# Firefox
command = ["firefox", "https://twitch.tv/{channel}"]
```

## Copyright
TWITCH, the TWITCH Logo, the Glitch Logo, and/or TWITCHTV are trademarks of Twitch Interactive, Inc. or its affiliates.

This project uses the Twitch API. Not affiliated with, endorsed by, or sponsored by Twitch
