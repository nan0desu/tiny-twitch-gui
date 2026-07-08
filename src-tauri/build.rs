fn main() {
    if std::env::var("TWITCH_CLIENT_ID").is_err() {
        panic!("TWITCH_CLIENT_ID is not set. Register at https://dev.twitch.tv/console");
    }
    println!("cargo:rerun-if-env-changed=TWITCH_CLIENT_ID");
    tauri_build::build()
}
