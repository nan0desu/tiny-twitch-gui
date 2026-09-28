mod api;
mod auth;
mod config;
mod session;

use auth::TokenData;
use config::Config;
use std::sync::Mutex;
use tauri::State;

const CLIENT_ID: &str = env!("TWITCH_CLIENT_ID");

struct AppState {
    token: Mutex<Option<TokenData>>,
    user_id: Mutex<Option<String>>,
    config: Mutex<Config>,
}

#[tauri::command]
async fn start_auth() -> Result<(), String> {
    let url = auth::build_auth_url(CLIENT_ID).map_err(|e| e.to_string())?;
    open::that(&url).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn finish_auth(state: State<'_, AppState>) -> Result<(), String> {
    let access_token = auth::wait_for_token().await.map_err(|e| e.to_string())?;

    let user_id = api::get_user_id(CLIENT_ID, &access_token)
        .await
        .map_err(|e| e.to_string())?;

    session::save(&session::SavedSession {
        access_token: access_token.clone(),
        user_id: user_id.clone(),
    })
    .map_err(|e| e.to_string())?;

    *state.token.lock().unwrap() = Some(TokenData { access_token });
    *state.user_id.lock().unwrap() = Some(user_id);

    Ok(())
}

#[tauri::command]
async fn get_streams(state: State<'_, AppState>) -> Result<Vec<api::Stream>, String> {
    let (token, user_id) = {
        let t = state.token.lock().unwrap().clone();
        let u = state.user_id.lock().unwrap().clone();
        match (t, u) {
            (Some(t), Some(u)) => (t, u),
            _ => return Err("not_authenticated".into()),
        }
    };

    // Scoped so the guard is dropped before the await below.
    let (thumb_w, thumb_h) = {
        let cfg = state.config.lock().unwrap();
        (cfg.thumbnails.width, cfg.thumbnails.height)
    };

    let streams =
        api::get_followed_streams(CLIENT_ID, &token.access_token, &user_id, thumb_w, thumb_h)
            .await
            .map_err(|e| e.to_string())?;

    Ok(streams)
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> config::Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn open_stream(channel: String, state: State<'_, AppState>) -> Result<(), String> {
    let cmd = state.config.lock().unwrap().resolve_command(&channel);
    let (bin, args) = cmd.split_first().ok_or("empty command in config")?;
    std::process::Command::new(bin)
        .args(args)
        .spawn()
        .map_err(|e| format!("failed to launch '{}': {}", bin, e))?;
    Ok(())
}

#[tauri::command]
async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    session::clear();
    *state.token.lock().unwrap() = None;
    *state.user_id.lock().unwrap() = None;
    Ok(())
}

/// Checks in-memory first, then tries to restore from disk (with token validation).
#[tauri::command]
async fn is_authenticated(state: State<'_, AppState>) -> Result<bool, String> {
    if state.token.lock().unwrap().is_some() {
        return Ok(true);
    }

    let Some(saved) = session::load() else {
        return Ok(false);
    };

    if !session::validate(&saved.access_token).await {
        session::clear();
        return Ok(false);
    }

    *state.token.lock().unwrap() = Some(TokenData {
        access_token: saved.access_token,
    });
    *state.user_id.lock().unwrap() = Some(saved.user_id);

    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::builder()
        .filter_module("twitch_live", log::LevelFilter::Debug)
        .init();

    let cfg = config::load().unwrap_or_else(|e| {
        eprintln!("config load error: {e}, using defaults");
        Config::default()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            token: Mutex::new(None),
            user_id: Mutex::new(None),
            config: Mutex::new(cfg),
        })
        .invoke_handler(tauri::generate_handler![
            start_auth,
            finish_auth,
            get_streams,
            get_config,
            open_stream,
            logout,
            is_authenticated,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
