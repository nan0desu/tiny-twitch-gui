use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Stream {
    pub user_name: String,
    pub user_login: String,
    pub title: String,
    pub viewer_count: u64,
    pub thumbnail_url: String,
    pub game_name: String,
    pub tags: Vec<String>,
    /// RFC 3339, as returned by Helix. Used by the UI to phase thumbnail refreshes.
    pub started_at: String,
}

#[derive(Deserialize)]
struct StreamsResponse {
    data: Vec<RawStream>,
}

#[derive(Deserialize)]
struct RawStream {
    user_name: String,
    user_login: String,
    title: String,
    viewer_count: u64,
    thumbnail_url: String,
    #[serde(default)]
    game_name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    started_at: String,
}

#[derive(Deserialize)]
struct UsersResponse {
    data: Vec<User>,
}

#[derive(Deserialize)]
struct User {
    id: String,
}

pub async fn get_user_id(client_id: &str, token: &str) -> Result<String> {
    log::debug!("GET /helix/users");
    let client = reqwest::Client::new();
    let resp: UsersResponse = client
        .get("https://api.twitch.tv/helix/users")
        .header("Client-Id", client_id)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let id = resp
        .data
        .into_iter()
        .next()
        .map(|u| u.id)
        .unwrap_or_default();
    log::debug!("got user_id={id}");
    Ok(id)
}

pub async fn get_followed_streams(
    client_id: &str,
    token: &str,
    user_id: &str,
    thumb_w: u32,
    thumb_h: u32,
) -> Result<Vec<Stream>> {
    log::debug!("GET /helix/streams/followed user_id={user_id}");
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.twitch.tv/helix/streams/followed")
        .query(&[("user_id", user_id), ("first", "50")])
        .header("Client-Id", client_id)
        .bearer_auth(token)
        .send()
        .await?;
    log::debug!("streams response status={}", response.status());
    let resp: StreamsResponse = response.error_for_status()?.json().await?;

    let streams: Vec<Stream> = resp
        .data
        .into_iter()
        .map(|s| Stream {
            user_name: s.user_name,
            user_login: s.user_login,
            title: s.title,
            viewer_count: s.viewer_count,
            thumbnail_url: s.thumbnail_url
                .replace("{width}", &thumb_w.to_string())
                .replace("{height}", &thumb_h.to_string()),
            game_name: s.game_name,
            tags: s.tags,
            started_at: s.started_at,
        })
        .collect();

    log::info!("streams/followed: {} live", streams.len());
    Ok(streams)
}
