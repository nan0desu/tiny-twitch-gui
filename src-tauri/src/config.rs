use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub thumbnails: ThumbnailConfig,
    pub player: PlayerConfig,
    pub notify: NotifyConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ThumbnailConfig {
    pub width: u32,
    pub height: u32,
    /// How often a thumbnail is re-fetched, in minutes. Counted from that stream's
    /// own start time, so cards reload at different moments instead of all at once.
    /// A stream younger than this is left alone. 0 disables refreshing.
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u32,
}

fn default_refresh_minutes() -> u32 {
    10
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerConfig {
    /// Command and args, e.g. ["mpv", "https://twitch.tv/{channel}"]
    /// Supported placeholders: {channel}, {url}
    pub command: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotifyConfig {
    /// Twitch logins to be notified about when they come online, matched
    /// case-insensitively. An empty list disables notifications.
    #[serde(default)]
    pub streamers: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thumbnails: ThumbnailConfig {
                width: 640,
                height: 360,
                refresh_minutes: default_refresh_minutes(),
            },
            player: PlayerConfig {
                command: vec!["streamlink".into(), "{channel}".into()],
            },
            notify: NotifyConfig {
                streamers: Vec::new(),
            },
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("twitchclient-live")
        .join("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        let cfg = Config::default();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(&cfg)?)?;
        return Ok(cfg);
    }

    let text = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&text)?)
}

impl Config {
    pub fn resolve_command(&self, channel: &str) -> Vec<String> {
        let url = format!("https://twitch.tv/{channel}");
        self.player
            .command
            .iter()
            .map(|arg| arg.replace("{channel}", channel).replace("{url}", &url))
            .collect()
    }
}
