mod github;

use github::{get_user, list_photos, poll_oauth, start_oauth, upload_photo, HttpClient};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(HttpClient::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_oauth,
            poll_oauth,
            get_user,
            upload_photo,
            list_photos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
