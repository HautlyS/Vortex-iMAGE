//! Rust Module - 1 functions, 0 structs
//! Core functionality: Backend operations and data processing
//! External crates: 4 dependencies

mod github;
mod compress;
mod crypto;
mod pipeline;

// Test modules - organized by functionality
#[cfg(test)]
mod tests;

use tauri::Manager;
use github::{
    get_user, list_photos, poll_oauth, start_oauth, upload_photo, validate_token,
    create_repo, get_repo_info, update_repo_visibility, scan_folder, upload_folder_as_album,
    upload_folder_recursive, list_albums, download_photo, delete_photo, remove_local_file,
    get_local_image_info, delete_album, rename_album, create_folder, HttpClient, download_secure_photo,
    upload_secure_message, download_secure_message, GithubConfig,
    check_keypair_sync, upload_keypair_sync, download_keypair_sync
};

use compress::{
    compress_data, compress_data_strict, decompress_data, estimate_compression, list_compression_algorithms,
    compress_data_auto, compress_file, decompress_file, get_compression_recommendation
};

use crypto::{
    generate_keypair, release_keypair, rotate_keypair, validate_keypair_handle,
    encrypt_data_password, decrypt_data_password,
    hash_data_blake3, get_crypto_info,
    encrypt_hybrid, decrypt_hybrid, sign_data, verify_signature,
    secure_store_token, secure_retrieve_token, secure_delete_token,
    encrypt_file, decrypt_file,
};

use pipeline::{
    pipeline_process, pipeline_reverse, pipeline_get_presets,
    pipeline_validate, pipeline_estimate
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(HttpClient::new())
        .setup(|_app| {
            // GitHub OAuth client ID - set via GITHUB_CLIENT_ID env var or use default
            let client_id = std::env::var("GITHUB_CLIENT_ID")
                .unwrap_or_else(|_| "Ov23lijNSMM1i93CQdfQ".to_string());
            _app.manage(GithubConfig { client_id });
            Ok(())
        })
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
            validate_token,
            
            upload_photo,
            list_photos,
            
            create_repo,
            get_repo_info,
            update_repo_visibility,
            
            scan_folder,
            upload_folder_as_album,
            upload_folder_recursive,
            list_albums,
            delete_album,
            rename_album,
            create_folder,
            
            download_photo,
            download_secure_photo,
            upload_secure_message,
            download_secure_message,
            delete_photo,
            remove_local_file,
            get_local_image_info,
            
            compress_data,
            compress_data_strict,
            compress_data_auto,
            decompress_data,
            estimate_compression,
            list_compression_algorithms,
            compress_file,
            decompress_file,
            get_compression_recommendation,
            
            generate_keypair,
            release_keypair,
            rotate_keypair,
            validate_keypair_handle,
            encrypt_data_password,
            decrypt_data_password,
            hash_data_blake3,
            get_crypto_info,
            
            encrypt_hybrid,
            decrypt_hybrid,
            
            sign_data,
            verify_signature,
            
            secure_store_token,
            secure_retrieve_token,
            secure_delete_token,
            
            encrypt_file,
            decrypt_file,
            
            // Keypair sync
            check_keypair_sync,
            upload_keypair_sync,
            download_keypair_sync,
            
            pipeline_process,
            pipeline_reverse,
            pipeline_get_presets,
            pipeline_validate,
            pipeline_estimate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}