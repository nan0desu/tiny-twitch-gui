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
    Ok(resp
        .data
        .into_iter()
        .next()
        .map(|u| u.id)
        .unwrap_or_default())
}

pub async fn get_followed_streams(
    client_id: &str,
    token: &str,
    user_id: &str,
    thumb_w: u32,
    thumb_h: u32,
) -> Result<Vec<Stream>> {
    let client = reqwest::Client::new();
    let resp: StreamsResponse = client
        .get("https://api.twitch.tv/helix/streams/followed")
        .query(&[("user_id", user_id), ("first", "50")])
        .header("Client-Id", client_id)
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let streams = resp
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
        })
        .collect();

    Ok(streams)
}
