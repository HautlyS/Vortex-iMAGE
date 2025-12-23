//! Rust Module - 1 functions, 0 structs
//! Core functionality: Backend operations and data processing
//! External crates: 4 dependencies

mod github;
mod compress;
mod crypto;
mod pipeline;
mod security_verify;

use github::{
    get_user, list_photos, poll_oauth, start_oauth, upload_photo,
    create_repo, get_repo_info, update_repo_visibility, scan_folder, upload_folder_as_album,
    upload_folder_recursive, list_albums, download_photo, delete_photo, remove_local_file,
    get_local_image_info, delete_album, rename_album, HttpClient, download_secure_photo,
    upload_secure_message, download_secure_message
};

use compress::{
    compress_data, compress_data_strict, decompress_data, estimate_compression, list_compression_algorithms,
    compress_data_auto, compress_file, decompress_file, get_compression_recommendation
};

use crypto::{
    generate_keypair, encrypt_data_password, decrypt_data_password,
    hash_data_blake3, get_crypto_info, encrypt_keypair, decrypt_keypair,
    encrypt_hybrid, decrypt_hybrid, sign_data, verify_signature, verify_signature_with_keypair,
    derive_session_keys, secure_store_token, secure_retrieve_token, encrypt_file, decrypt_file,
    check_pqcrypto_backend, require_optimized_backend
};

use pipeline::{
    pipeline_process, pipeline_reverse, pipeline_get_presets,
    pipeline_validate, pipeline_estimate
};

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
            encrypt_data_password,
            decrypt_data_password,
            hash_data_blake3,
            get_crypto_info,
            
            encrypt_keypair,
            decrypt_keypair,
            
            encrypt_hybrid,
            decrypt_hybrid,
            
            sign_data,
            verify_signature,
            verify_signature_with_keypair,
            
            derive_session_keys,
            
            secure_store_token,
            secure_retrieve_token,
            
            encrypt_file,
            decrypt_file,
            
            check_pqcrypto_backend,
            require_optimized_backend,
            
            pipeline_process,
            pipeline_reverse,
            pipeline_get_presets,
            pipeline_validate,
            pipeline_estimate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}