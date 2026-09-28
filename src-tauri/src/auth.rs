use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

const REDIRECT_PORT: u16 = 17563;

pub fn redirect_uri() -> String {
    format!("http://localhost:{}/callback", REDIRECT_PORT)
}

pub fn build_auth_url(client_id: &str) -> Result<String> {
    let redirect = redirect_uri();
    let url = url::Url::parse_with_params(
        "https://id.twitch.tv/oauth2/authorize",
        &[
            ("client_id", client_id),
            ("redirect_uri", redirect.as_str()),
            ("response_type", "token"),
            ("scope", "user:read:follows"),
        ],
    )?;
    Ok(url.into())
}

pub async fn wait_for_token() -> Result<String> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", REDIRECT_PORT)).await?;

    // First request: Twitch redirects here with #access_token=... in the fragment.
    // Fragments are not sent to servers, so we serve a page that forwards it as a query param.
    let (mut stream, _) = listener.accept().await?;
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    if path.starts_with("/callback") {
        let html = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
            <script>\
            const p=new URLSearchParams(location.hash.slice(1));\
            const t=p.get('access_token');\
            if(t)location.replace('http://localhost:{port}/token?access_token='+t);\
            </script>",
            port = REDIRECT_PORT
        );
        stream.write_all(html.as_bytes()).await?;
        drop(stream);

        // Second request: browser navigates to /token?access_token=...
        let (mut stream2, _) = listener.accept().await?;
        let mut reader2 = BufReader::new(&mut stream2);
        let mut line2 = String::new();
        reader2.read_line(&mut line2).await?;

        let path2 = line2.split_whitespace().nth(1).unwrap_or("");
        let query = path2.splitn(2, '?').nth(1).unwrap_or("");
        let params: HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes()).collect();

        let token = params
            .get("access_token")
            .ok_or_else(|| anyhow!("no access_token in callback"))?
            .to_string();

        let done = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
            <html><meta charset='utf-8'><body style='font-family:sans-serif;text-align:center;padding:4rem'>\
            <h2>Авторизация прошла успешно!</h2>\
            <p>Можно закрыть это окно.</p></body></html>";
        stream2.write_all(done.as_bytes()).await?;

        Ok(token)
    } else {
        Err(anyhow!("unexpected callback path: {}", path))
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct TokenData {
    pub access_token: String,
}
