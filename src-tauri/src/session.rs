use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct SavedSession {
    pub access_token: String,
    pub user_id: String,
}

fn session_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("twitchclient-live")
        .join("session.json")
}

pub fn save(session: &SavedSession) -> Result<()> {
    let path = session_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string(session)?)?;
    Ok(())
}

pub fn load() -> Option<SavedSession> {
    let text = std::fs::read_to_string(session_path()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn clear() {
    let _ = std::fs::remove_file(session_path());
}

pub async fn validate(token: &str) -> bool {
    let client = reqwest::Client::new();
    client
        .get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(token)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
