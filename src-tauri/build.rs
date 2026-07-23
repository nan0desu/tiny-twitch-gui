fn main() {
    let _ = dotenvy::from_filename("../.env");

    let client_id = std::env::var("TWITCH_CLIENT_ID")
        .expect("TWITCH_CLIENT_ID is not set. Register at https://dev.twitch.tv/console");

    println!("cargo:rustc-env=TWITCH_CLIENT_ID={client_id}");
    println!("cargo:rerun-if-env-changed=TWITCH_CLIENT_ID");
    println!("cargo:rerun-if-changed=../.env");
    tauri_build::build()
}
