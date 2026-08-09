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
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerConfig {
    /// Command and args, e.g. ["mpv", "https://twitch.tv/{channel}"]
    /// Supported placeholders: {channel}, {url}
    pub command: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NotifyConfig {
    pub notify: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thumbnails: ThumbnailConfig {
                width: 440,
                height: 248,
            },
            player: PlayerConfig {
                command: vec!["streamlink".into(), "{channel}".into()],
            },
            notify: NotifyConfig {
                notify: String::from(""),
            },
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("twitch-live")
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
