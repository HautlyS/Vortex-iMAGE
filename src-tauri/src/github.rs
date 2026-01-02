//! Rust Module - 40 functions, 16 structs
//! Core functionality: Backend operations and data processing
//! External crates: 15 dependencies

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::fs;
use tokio::time::sleep;

use crate::compress::{compress_file_data, ItemCompressionSettings, Algorithm};
use crate::crypto::{encrypt, decrypt_with_keypair_bytes, PublicBundle, EncryptedFileData, EncryptionSettings, EncryptedPayload};

pub struct GithubConfig {
    pub client_id: String,
}

const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 1000;
const UPLOAD_TIMEOUT_SECS: u64 = 120;
const LFS_UPLOAD_TIMEOUT_SECS: u64 = 300;
const LFS_THRESHOLD_BYTES: u64 = 50 * 1024 * 1024;
const HTTP_POOL_SIZE: usize = 5;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("API error: {0}")]
    Api(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub struct HttpClient(pub Arc<Client>);

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(HTTP_POOL_SIZE)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        Self(Arc::new(client))
    }

    #[inline]
    #[allow(dead_code)]
    pub fn inner(&self) -> &Client {
        &self.0
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = initial_delay_ms;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(e);
                }
                
                let jitter = rand::random::<u64>() % (delay / 2);
                sleep(Duration::from_millis(delay + jitter)).await;
                delay *= 2;
            }
        }
    }
}

#[inline]
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::TOO_MANY_REQUESTS
            | reqwest::StatusCode::INTERNAL_SERVER_ERROR
            | reqwest::StatusCode::BAD_GATEWAY
            | reqwest::StatusCode::SERVICE_UNAVAILABLE
            | reqwest::StatusCode::GATEWAY_TIMEOUT
            | reqwest::StatusCode::REQUEST_TIMEOUT
    )
}

fn get_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

#[derive(Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadResult {
    pub url: String,
    pub sha: String,
}

#[derive(Serialize, Clone)]
pub struct UploadProgress {
    pub id: String,
    pub bytes_sent: u64,
    pub total_bytes: u64,
    pub percent: u8,
}

fn validate_repo(repo: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 || parts.iter().any(|p| p.is_empty() || p.contains("..")) {
        return Err(AppError::Validation("Invalid repo format. Use owner/repo".into()));
    }
    Ok(())
}

fn sanitize_filename(name: &str) -> String {
    name.replace("..", "")
        .replace('/', "_")
        .replace('\\', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect()
}

#[tauri::command]
pub async fn start_oauth(
    client: State<'_, HttpClient>,
    config: State<'_, GithubConfig>,
) -> Result<DeviceCodeResponse, AppError> {
    let res = client
        .0
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", config.client_id.as_str()), ("scope", "repo")])
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Api(format!("OAuth failed: {}", res.status())));
    }

    Ok(res.json().await?)
}

#[tauri::command]
pub async fn poll_oauth(
    client: State<'_, HttpClient>,
    device_code: String,
    config: State<'_, GithubConfig>,
) -> Result<Option<String>, AppError> {
    let res = client
        .0
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("device_code", device_code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await?;

    let token_res: TokenResponse = res.json().await?;

    if let Some(ref err) = token_res.error {
        if err == "authorization_pending" || err == "slow_down" {
            return Ok(None);
        }
        return Err(AppError::Api(err.clone()));
    }

    Ok(token_res.access_token)
}

#[tauri::command]
pub async fn get_user(
    client: State<'_, HttpClient>,
    token: String,
) -> Result<GitHubUser, AppError> {
    let res = client
        .0
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Api("Invalid token".into()));
    }

    Ok(res.json().await?)
}

#[derive(Serialize, Deserialize)]
pub struct TokenValidation {
    pub valid: bool,
    pub user: Option<GitHubUser>,
    pub scopes: Vec<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn validate_token(
    client: State<'_, HttpClient>,
    token: String,
) -> Result<TokenValidation, AppError> {
    let res = client
        .0
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .send()
        .await?;

    if !res.status().is_success() {
        return Ok(TokenValidation {
            valid: false,
            user: None,
            scopes: vec![],
            error: Some("Invalid or expired token".into()),
        });
    }

    let scopes: Vec<String> = res
        .headers()
        .get("x-oauth-scopes")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(", ").map(|s| s.to_string()).collect())
        .unwrap_or_default();

    let user: GitHubUser = res.json().await?;

    let has_repo = scopes.iter().any(|s| s == "repo" || s == "public_repo");
    
    if !has_repo {
        return Ok(TokenValidation {
            valid: false,
            user: Some(user),
            scopes,
            error: Some("Token missing 'repo' scope. Generate a new token with repo access.".into()),
        });
    }

    Ok(TokenValidation {
        valid: true,
        user: Some(user),
        scopes,
        error: None,
    })
}

async fn prepare_upload_payload(
    content: &[u8],
    filename: &str,
    public_bundle: PublicBundle,
    app: &AppHandle,
    upload_id: &str,
) -> Result<Vec<u8>, AppError> {
    let total_bytes = content.len() as u64;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes,
        percent: 0,
    });

    let compression_settings = ItemCompressionSettings {
        enabled: true,
        algorithm: Algorithm::Zstd,
        level: 3,
        prefer_speed: false,
        min_size_threshold: 128,
        skip_already_compressed: false,
    };

    let compressed_data = compress_file_data(content, filename, &compression_settings)
        .map_err(|e| AppError::Validation(format!("Compression failed: {}", e)))?;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes,
        percent: 30,
    });

    let compressed_bytes = serde_json::to_vec(&compressed_data)
        .map_err(|e| AppError::Validation(format!("Serialization failed: {}", e)))?;
    drop(compressed_data);

    // Note: encryption_settings kept for documentation/future use
    let _encryption_settings = EncryptionSettings {
        enabled: true,
        use_password: false,
        use_keypair: true,
        recipient_bundle: Some(public_bundle.clone()),
    };

    // Encrypt using hybrid PQ encryption
    let encrypted_payload = encrypt(&compressed_bytes, &public_bundle)
        .map_err(|e| AppError::Validation(format!("Encryption failed: {}", e)))?;
    drop(compressed_bytes);

    // Wrap in EncryptedFileData for compatibility
    let encrypted_file = EncryptedFileData {
        data: serde_json::to_vec(&encrypted_payload)
            .map_err(|e| AppError::Validation(format!("Payload serialization failed: {}", e)))?,
        encrypted: true,
        method: crate::crypto::EncryptionMethod::HybridPQ,
        metadata: None,
    };
    drop(encrypted_payload);
        
    let final_payload = serde_json::to_vec(&encrypted_file)
        .map_err(|e| AppError::Validation(format!("Final serialization failed: {}", e)))?;
    drop(encrypted_file);

    let final_size = final_payload.len() as u64;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes: final_size,
        percent: 60,
    });

    Ok(final_payload)
}

#[tauri::command]
pub async fn upload_photo(
    app: AppHandle,
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
    filename: String,
    upload_id: String,
    public_bundle: PublicBundle,
) -> Result<UploadResult, AppError> {
    validate_repo(&repo)?;
    let safe_filename = sanitize_filename(&filename);

    if safe_filename.is_empty() {
        return Err(AppError::Validation("Invalid filename".into()));
    }

    let content = fs::read(&path).await?;

    let final_payload =
        prepare_upload_payload(&content, &safe_filename, public_bundle, &app, &upload_id).await?;

    upload_to_github(
        &app,
        &client.0,
        final_payload,
        &repo,
        &token,
        &safe_filename,
        &upload_id,
    )
    .await
}

async fn upload_to_github(
    app: &AppHandle,
    client: &Client,
    payload: Vec<u8>,
    repo: &str,
    token: &str,
    filename: &str,
    upload_id: &str,
) -> Result<UploadResult, AppError> {
    let final_size = payload.len() as u64;

    if final_size > LFS_THRESHOLD_BYTES {
        return upload_lfs_internal(app, client, payload, repo, token, filename, upload_id).await;
    }

    let encoded = STANDARD.encode(&payload);
    drop(payload);

    let upload_path = format!("photos/{}", filename);
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, upload_path);

    let body = serde_json::json!({
        "message": format!("Upload {} (secure)", filename),
        "content": encoded
    });

    let res = client
        .put(&url)
        .timeout(Duration::from_secs(UPLOAD_TIMEOUT_SECS))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: final_size,
        total_bytes: final_size,
        percent: 100,
    });

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.map_err(|e| AppError::Api(format!("Failed to read error response body: {}", e)))?;
        return Err(AppError::Api(format!("Upload failed ({}): {}", status, err_text)));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(UploadResult {
        url: json["content"]["html_url"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain html_url".to_string()))?.to_string(),
        sha: json["content"]["sha"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain sha".to_string()))?.to_string(),
    })
}

#[allow(dead_code)]
fn process_image(content: &[u8], strip_exif: bool, compress: bool, quality: u8) -> Result<Vec<u8>, AppError> {
    
    let img = image::load_from_memory(content)
        .map_err(|e| AppError::Validation(format!("Failed to decode image: {}", e)))?;

    let format = image::guess_format(content)
        .unwrap_or(ImageFormat::Jpeg);

    let mut output = Cursor::new(Vec::new());

    if compress || strip_exif {
        
        match format {
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
                img.write_with_encoder(encoder)
                    .map_err(|e| AppError::Validation(format!("Failed to encode JPEG: {}", e)))?;
            }
            ImageFormat::Png => {
                img.write_to(&mut output, ImageFormat::Png)
                    .map_err(|e| AppError::Validation(format!("Failed to encode PNG: {}", e)))?;
            }
            ImageFormat::WebP => {
                img.write_to(&mut output, ImageFormat::WebP)
                    .map_err(|e| AppError::Validation(format!("Failed to encode WebP: {}", e)))?;
            }
            _ => {
                
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
                img.write_with_encoder(encoder)
                    .map_err(|e| AppError::Validation(format!("Failed to encode image: {}", e)))?;
            }
        }
        Ok(output.into_inner())
    } else {
        
        Ok(content.to_vec())
    }
}

async fn upload_lfs_internal(
    app: &AppHandle,
    client: &Client,
    content: Vec<u8>,
    repo: &str,
    token: &str,
    filename: &str,
    upload_id: &str,
) -> Result<UploadResult, AppError> {
    let total_bytes = content.len() as u64;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes,
        percent: 10,
    });

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let oid = format!("{:x}", hasher.finalize());

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes,
        percent: 20,
    });

    let batch_url = format!("https://github.com/{}.git/info/lfs/objects/batch", repo);
    let batch_body = serde_json::json!({
        "operation": "upload",
        "transfers": ["basic"],
        "objects": [{ "oid": oid, "size": total_bytes }]
    });

    let batch_res = client
        .post(&batch_url)
        .timeout(std::time::Duration::from_secs(60))
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.git-lfs+json")
        .header("Content-Type", "application/vnd.git-lfs+json")
        .json(&batch_body)
        .send()
        .await?;

    if !batch_res.status().is_success() {
        return Err(AppError::Api(format!("LFS batch failed: {}", batch_res.status())));
    }

    let batch_json: serde_json::Value = batch_res.json().await?;

    let upload_href = batch_json["objects"][0]["actions"]["upload"]["href"]
        .as_str()
        .ok_or_else(|| AppError::Api("No LFS upload URL returned".into()))?;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: 0,
        total_bytes,
        percent: 30,
    });

    let upload_res = client
        .put(upload_href)
        .timeout(Duration::from_secs(LFS_UPLOAD_TIMEOUT_SECS))
        .header("Content-Type", "application/octet-stream")
        .body(content)
        .send()
        .await?;

    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.to_string(),
        bytes_sent: total_bytes,
        total_bytes,
        percent: 100,
    });

    if !upload_res.status().is_success() {
        return Err(AppError::Api(format!("LFS upload failed: {}", upload_res.status())));
    }

    Ok(UploadResult {
        url: format!("https://github.com/{}/blob/main/photos/{}", repo, filename),
        sha: oid,
    })
}

#[derive(Serialize, Deserialize)]
pub struct PhotoItem {
    pub name: String,
    pub url: String,
    pub sha: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub default_branch: String,
}

pub fn validate_repo_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::Validation(
            "Repository name must be 1-100 characters".into(),
        ));
    }

    let valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    if !valid {
        return Err(AppError::Validation(
            "Repository name can only contain letters, numbers, hyphens, underscores, and dots"
                .into(),
        ));
    }

    if name.starts_with('.') || name.starts_with('-') || name.ends_with('.') || name.ends_with('-')
    {
        return Err(AppError::Validation(
            "Repository name cannot start or end with a dot or hyphen".into(),
        ));
    }

    if name.contains("..") {
        return Err(AppError::Validation(
            "Repository name cannot contain consecutive dots".into(),
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn create_repo(
    client: State<'_, HttpClient>,
    token: String,
    name: String,
    description: String,
    private: bool,
) -> Result<RepoInfo, AppError> {
    validate_repo_name(&name)?;

    let body = serde_json::json!({
        "name": name,
        "description": description,
        "private": private,
        "auto_init": true
    });

    let res = client
        .0
        .post("https://api.github.com/user/repos")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.map_err(|e| AppError::Api(format!("Failed to read error response body: {}", e)))?;
        return Err(AppError::Api(format!(
            "Failed to create repository ({}): {}",
            status, err_text
        )));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(RepoInfo {
        name: json["name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain name".to_string()))?.to_string(),
        full_name: json["full_name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain full_name".to_string()))?.to_string(),
        private: json["private"].as_bool().ok_or_else(|| AppError::Validation("GitHub API response did not contain private".to_string()))?,
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain html_url".to_string()))?.to_string(),
        default_branch: json["default_branch"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain default_branch".to_string()))?.to_string(),
    })
}

#[tauri::command]
pub async fn get_repo_info(
    client: State<'_, HttpClient>,
    token: String,
    repo: String,
) -> Result<RepoInfo, AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}", repo);

    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        return Err(AppError::Api(format!(
            "Failed to get repository info: {}",
            status
        )));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(RepoInfo {
        name: json["name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain name".to_string()))?.to_string(),
        full_name: json["full_name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain full_name".to_string()))?.to_string(),
        private: json["private"].as_bool().ok_or_else(|| AppError::Validation("GitHub API response did not contain private".to_string()))?,
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain html_url".to_string()))?.to_string(),
        default_branch: json["default_branch"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain default_branch".to_string()))?.to_string(),
    })
}

#[tauri::command]
pub async fn update_repo_visibility(
    client: State<'_, HttpClient>,
    token: String,
    repo: String,
    private: bool,
) -> Result<RepoInfo, AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}", repo);

    let body = serde_json::json!({
        "private": private
    });

    let res = client
        .0
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.map_err(|e| AppError::Api(format!("Failed to read error response body: {}", e)))?;
        return Err(AppError::Api(format!(
            "Failed to update repository visibility ({}): {}",
            status, err_text
        )));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(RepoInfo {
        name: json["name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain name".to_string()))?.to_string(),
        full_name: json["full_name"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain full_name".to_string()))?.to_string(),
        private: json["private"].as_bool().ok_or_else(|| AppError::Validation("GitHub API response did not contain private".to_string()))?,
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain html_url".to_string()))?.to_string(),
        default_branch: json["default_branch"].as_str().ok_or_else(|| AppError::Validation("GitHub API response did not contain default_branch".to_string()))?.to_string(),
    })
}

#[tauri::command]
pub async fn list_photos(
    client: State<'_, HttpClient>,
    repo: String,
    token: String,
    folder: Option<String>,
) -> Result<Vec<PhotoItem>, AppError> {
    validate_repo(&repo)?;
    
    let folder_path = folder.unwrap_or_else(|| "photos".to_string());
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, folder_path);

    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .send()
        .await?;

    if res.status() == 404 {
        return Ok(vec![]);
    }

    if !res.status().is_success() {
        return Err(AppError::Api(format!("Failed to list photos: {}", res.status())));
    }

    let json: Vec<serde_json::Value> = res.json().await?;

    Ok(json
        .iter()
        .filter_map(|f| {
            Some(PhotoItem {
                name: f["name"].as_str()?.to_string(),
                url: f["download_url"].as_str()?.to_string(),
                sha: f["sha"].as_str()?.to_string(),
            })
        })
        .collect())
}

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "svg", "ico", "heic", "heif", "avif",
];

fn is_image_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FolderScanResult {
    pub path: String,
    pub name: String,
    pub image_count: usize,
    pub total_size: u64,
    pub subfolders: Vec<FolderScanResult>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ImageFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub relative_path: String,
}

#[derive(Serialize, Clone)]
pub struct UploadBatchProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file: String,
    pub percent: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadBatchResult {
    pub succeeded: Vec<UploadResult>,
    pub failed: Vec<UploadFailure>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UploadFailure {
    pub path: String,
    pub name: String,
    pub error: String,
}

#[tauri::command]
pub async fn scan_folder(path: String) -> Result<FolderScanResult, AppError> {
    let folder_path = std::path::Path::new(&path);

    if !folder_path.exists() {
        return Err(AppError::Validation("Folder does not exist".into()));
    }

    if !folder_path.is_dir() {
        return Err(AppError::Validation("Path is not a directory".into()));
    }

    scan_folder_recursive(folder_path).await
}

async fn scan_folder_recursive(folder_path: &std::path::Path) -> Result<FolderScanResult, AppError> {
    let name = folder_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut image_count = 0;
    let mut total_size = 0u64;
    let mut subfolders = Vec::new();

    let mut entries = fs::read_dir(folder_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let metadata = entry.metadata().await?;

        if metadata.is_dir() {
            
            if entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            let subfolder = Box::pin(scan_folder_recursive(&entry_path)).await?;
            subfolders.push(subfolder);
        } else if metadata.is_file() && is_image_file(&entry_path) {
            image_count += 1;
            total_size += metadata.len();
        }
    }

    Ok(FolderScanResult {
        path: folder_path.to_string_lossy().to_string(),
        name,
        image_count,
        total_size,
        subfolders,
    })
}

async fn collect_images_in_folder(folder_path: &std::path::Path) -> Result<Vec<ImageFile>, AppError> {
    let mut images = Vec::new();
    let mut entries = fs::read_dir(folder_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let metadata = entry.metadata().await?;

        if metadata.is_file() && is_image_file(&entry_path) {
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            images.push(ImageFile {
                path: entry_path.to_string_lossy().to_string(),
                name: name.clone(),
                size: metadata.len(),
                relative_path: name,
            });
        }
    }

    Ok(images)
}

async fn collect_images_recursive(
    folder_path: &std::path::Path,
    base_path: &std::path::Path,
) -> Result<Vec<ImageFile>, AppError> {
    let mut images = Vec::new();
    let mut entries = fs::read_dir(folder_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        let metadata = entry.metadata().await?;

        if metadata.is_dir() {
            
            if entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            let mut sub_images = Box::pin(collect_images_recursive(&entry_path, base_path)).await?;
            images.append(&mut sub_images);
        } else if metadata.is_file() && is_image_file(&entry_path) {
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let relative = entry_path
                .strip_prefix(base_path)
                .unwrap_or(&entry_path)
                .to_string_lossy()
                .to_string();

            images.push(ImageFile {
                path: entry_path.to_string_lossy().to_string(),
                name,
                size: metadata.len(),
                relative_path: relative,
            });
        }
    }

    Ok(images)
}

#[tauri::command]
pub async fn upload_folder_as_album(
    app: AppHandle,
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
    album_name: String,
    create_subalbums: bool,
) -> Result<UploadBatchResult, AppError> {
    validate_repo(&repo)?;

    let folder_path = std::path::Path::new(&path);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err(AppError::Validation("Invalid folder path".into()));
    }

    let safe_album_name = sanitize_filename(&album_name);
    if safe_album_name.is_empty() {
        return Err(AppError::Validation("Invalid album name".into()));
    }

    let images = if create_subalbums {
        collect_images_recursive(folder_path, folder_path).await?
    } else {
        collect_images_in_folder(folder_path).await?
    };

    let total_files = images.len();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, image) in images.iter().enumerate() {
        
        let _ = app.emit(
            "batch-upload-progress",
            UploadBatchProgress {
                total_files,
                completed_files: index,
                current_file: image.name.clone(),
                percent: ((index * 100) / total_files.max(1)) as u8,
            },
        );

        let upload_path = if create_subalbums {
            format!("photos/{}/{}", safe_album_name, image.relative_path.replace('\\', "/"))
        } else {
            format!("photos/{}/{}", safe_album_name, image.name)
        };

        match upload_single_file(&client.0, &image.path, &repo, &token, &upload_path).await {
            Ok(result) => succeeded.push(result),
            Err(e) => failed.push(UploadFailure {
                path: image.path.clone(),
                name: image.name.clone(),
                error: e.to_string(),
            }),
        }
    }

    let _ = app.emit(
        "batch-upload-progress",
        UploadBatchProgress {
            total_files,
            completed_files: total_files,
            current_file: String::new(),
            percent: 100,
        },
    );

    Ok(UploadBatchResult { succeeded, failed })
}

#[tauri::command]
pub async fn upload_folder_recursive(
    app: AppHandle,
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
) -> Result<UploadBatchResult, AppError> {
    validate_repo(&repo)?;

    let folder_path = std::path::Path::new(&path);
    if !folder_path.exists() || !folder_path.is_dir() {
        return Err(AppError::Validation("Invalid folder path".into()));
    }

    let images = collect_images_recursive(folder_path, folder_path).await?;

    let total_files = images.len();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, image) in images.iter().enumerate() {
        
        let _ = app.emit(
            "batch-upload-progress",
            UploadBatchProgress {
                total_files,
                completed_files: index,
                current_file: image.name.clone(),
                percent: ((index * 100) / total_files.max(1)) as u8,
            },
        );

        let safe_name = sanitize_filename(&image.name);
        let upload_path = format!("photos/{}", safe_name);

        match upload_single_file(&client.0, &image.path, &repo, &token, &upload_path).await {
            Ok(result) => succeeded.push(result),
            Err(e) => failed.push(UploadFailure {
                path: image.path.clone(),
                name: image.name.clone(),
                error: e.to_string(),
            }),
        }
    }

    let _ = app.emit(
        "batch-upload-progress",
        UploadBatchProgress {
            total_files,
            completed_files: total_files,
            current_file: String::new(),
            percent: 100,
        },
    );

    Ok(UploadBatchResult { succeeded, failed })
}

async fn upload_single_file(
    client: &Client,
    local_path: &str,
    repo: &str,
    token: &str,
    upload_path: &str,
) -> Result<UploadResult, AppError> {
    let content = fs::read(local_path).await?;
    let encoded = STANDARD.encode(&content);

    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, upload_path);

    let body = serde_json::json!({
        "message": format!("Upload {}", upload_path),
        "content": encoded
    });

    let result = retry_with_backoff(
        || async {
            let res = client
                .put(&url)
                .timeout(Duration::from_secs(UPLOAD_TIMEOUT_SECS))
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "vortex-image")
                .header("Accept", "application/vnd.github+json")
                .json(&body)
                .send()
                .await?;

            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                
                if let Some(retry_secs) = get_retry_after(res.headers()) {
                    sleep(Duration::from_secs(retry_secs)).await;
                }
                return Err(AppError::Api("Rate limited".into()));
            }

            if is_retryable_status(res.status()) {
                return Err(AppError::Api(format!("Retryable error: {}", res.status())));
            }

            if !res.status().is_success() {
                let status = res.status();
                let err = res.text().await.unwrap_or_default();
                return Err(AppError::Api(format!("Upload failed ({}): {}", status, err)));
            }

            let json: serde_json::Value = res.json().await?;
            Ok(UploadResult {
                url: json["content"]["html_url"].as_str().unwrap_or("").to_string(),
                sha: json["content"]["sha"].as_str().unwrap_or("").to_string(),
            })
        },
        MAX_RETRIES,
        INITIAL_RETRY_DELAY_MS,
    )
    .await?;

    Ok(result)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Album {
    pub name: String,
    pub path: String,
    pub photo_count: usize,
    pub children: Vec<Album>,
}

#[tauri::command]
pub async fn list_albums(
    client: State<'_, HttpClient>,
    repo: String,
    token: String,
) -> Result<Vec<Album>, AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}/contents/photos", repo);

    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if res.status() == 404 {
        return Ok(vec![]);
    }

    if !res.status().is_success() {
        return Err(AppError::Api(format!("Failed to list albums: {}", res.status())));
    }

    let items: Vec<serde_json::Value> = res.json().await?;

    let mut albums = Vec::new();

    for item in items {
        if item["type"].as_str() == Some("dir") {
            let name = item["name"].as_str().unwrap_or("").to_string();
            let path = item["path"].as_str().unwrap_or("").to_string();

            let album = get_album_recursive(&client.0, &repo, &token, &path, &name).await?;
            albums.push(album);
        }
    }

    Ok(albums)
}

async fn get_album_recursive(
    client: &Client,
    repo: &str,
    token: &str,
    path: &str,
    name: &str,
) -> Result<Album, AppError> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Ok(Album {
            name: name.to_string(),
            path: path.to_string(),
            photo_count: 0,
            children: vec![],
        });
    }

    let items: Vec<serde_json::Value> = res.json().await?;

    let mut photo_count = 0;
    let mut children = Vec::new();

    for item in items {
        let item_type = item["type"].as_str().unwrap_or("");
        let item_name = item["name"].as_str().unwrap_or("");

        if item_type == "file" {
            
            let ext = std::path::Path::new(item_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                photo_count += 1;
            }
        } else if item_type == "dir" {
            let child_path = item["path"].as_str().unwrap_or("").to_string();
            let child = Box::pin(get_album_recursive(client, repo, token, &child_path, item_name)).await?;
            children.push(child);
        }
    }

    Ok(Album {
        name: name.to_string(),
        path: path.to_string(),
        photo_count,
        children,
    })
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub id: String,
    pub bytes_received: u64,
    pub total_bytes: u64,
    pub percent: u8,
}

#[tauri::command]
pub async fn download_photo(
    app: AppHandle,
    client: State<'_, HttpClient>,
    remote_path: String,
    repo: String,
    token: String,
    download_id: String,
    local_dir: Option<String>,
) -> Result<String, AppError> {
    validate_repo(&repo)?;

    let _ = app.emit("download-progress", DownloadProgress {
        id: download_id.clone(),
        bytes_received: 0,
        total_bytes: 0,
        percent: 0,
    });

    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, remote_path);
    
    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Api(format!("Failed to get file info: {}", res.status())));
    }

    let json: serde_json::Value = res.json().await?;
    let download_url = json["download_url"]
        .as_str()
        .ok_or_else(|| AppError::Api("No download URL found".into()))?;

    let content_res = client
        .0
        .get(download_url)
        .header("User-Agent", "vortex-image")
        .send()
        .await?;

    if !content_res.status().is_success() {
        return Err(AppError::Api(format!("Failed to download file: {}", content_res.status())));
    }

    let total_bytes = content_res.content_length().unwrap_or(0);
    let content = content_res.bytes().await?;

    let _ = app.emit("download-progress", DownloadProgress {
        id: download_id.clone(),
        bytes_received: content.len() as u64,
        total_bytes,
        percent: 100,
    });

    let filename = remote_path.split('/').last().unwrap_or("photo");
    let local_path = if let Some(dir) = local_dir {
        std::path::Path::new(&dir).join(filename)
    } else {
        
        let downloads = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        downloads.join(filename)
    };

    fs::write(&local_path, &content).await?;

    Ok(local_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn delete_photo(
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
) -> Result<(), AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
    
    let get_res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !get_res.status().is_success() {
        return Err(AppError::Api(format!("File not found: {}", get_res.status())));
    }

    let json: serde_json::Value = get_res.json().await?;
    let sha = json["sha"]
        .as_str()
        .ok_or_else(|| AppError::Api("Could not get file SHA".into()))?;

    let delete_body = serde_json::json!({
        "message": format!("Delete {}", path),
        "sha": sha
    });

    let delete_res = client
        .0
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&delete_body)
        .send()
        .await?;

    if !delete_res.status().is_success() {
        let status = delete_res.status();
        let err_text = delete_res.text().await.map_err(|e| AppError::Api(format!("Failed to read error response body: {}", e)))?;
        return Err(AppError::Api(format!("Failed to delete file ({}): {}", status, err_text)));
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_local_file(path: String) -> Result<(), AppError> {
    let file_path = std::path::Path::new(&path);
    
    if !file_path.exists() {
        return Err(AppError::Validation("File does not exist".into()));
    }

    if !file_path.is_file() {
        return Err(AppError::Validation("Path is not a file".into()));
    }

    fs::remove_file(file_path).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_local_image_info(path: String) -> Result<ImageFile, AppError> {
    let file_path = std::path::Path::new(&path);
    
    if !file_path.exists() {
        return Err(AppError::Validation("File does not exist".into()));
    }

    let metadata = fs::metadata(file_path).await?;
    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(ImageFile {
        path: path.clone(),
        name,
        size: metadata.len(),
        relative_path: path,
    })
}

#[tauri::command]
pub async fn delete_album(
    client: State<'_, HttpClient>,
    album_path: String,
    repo: String,
    token: String,
) -> Result<u32, AppError> {
    validate_repo(&repo)?;

    let files = get_album_files_recursive(&client.0, &repo, &token, &album_path).await?;
    
    if files.is_empty() {
        return Err(AppError::Validation("Album is empty or does not exist".into()));
    }

    let mut deleted_count = 0u32;

    for file in files {
        let url = format!("https://api.github.com/repos/{}/contents/{}", repo, file.path);

        let get_res = client
            .0
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !get_res.status().is_success() {
            continue; 
        }

        let json: serde_json::Value = get_res.json().await?;
        let sha = match json["sha"].as_str() {
            Some(s) => s,
            None => continue,
        };

        let delete_body = serde_json::json!({
            "message": format!("Delete {} (album cleanup)", file.path),
            "sha": sha
        });

        let delete_res = client
            .0
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .json(&delete_body)
            .send()
            .await?;

        if delete_res.status().is_success() {
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

#[derive(Clone)]
struct FileInfo {
    path: String,
}

async fn get_album_files_recursive(
    client: &Client,
    repo: &str,
    token: &str,
    path: &str,
) -> Result<Vec<FileInfo>, AppError> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Ok(vec![]);
    }

    let items: Vec<serde_json::Value> = res.json().await?;
    let mut files = Vec::new();

    for item in items {
        let item_type = item["type"].as_str().unwrap_or("");
        let item_path = item["path"].as_str().unwrap_or("").to_string();

        if item_type == "file" {
            files.push(FileInfo { path: item_path });
        } else if item_type == "dir" {
            let mut sub_files = Box::pin(get_album_files_recursive(client, repo, token, &item_path)).await?;
            files.append(&mut sub_files);
        }
    }

    Ok(files)
}

#[tauri::command]
pub async fn rename_album(
    client: State<'_, HttpClient>,
    old_path: String,
    new_name: String,
    repo: String,
    token: String,
) -> Result<u32, AppError> {
    validate_repo(&repo)?;
    
    let safe_new_name = sanitize_filename(&new_name);
    if safe_new_name.is_empty() {
        return Err(AppError::Validation("Invalid album name".into()));
    }

    let parent = std::path::Path::new(&old_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let new_path = if parent.is_empty() {
        safe_new_name.clone()
    } else {
        format!("{}/{}", parent, safe_new_name)
    };

    let files = get_album_files_recursive(&client.0, &repo, &token, &old_path).await?;
    
    if files.is_empty() {
        return Err(AppError::Validation("Album is empty or does not exist".into()));
    }

    let mut moved_count = 0u32;

    for file in files {
        
        let relative = file.path.strip_prefix(&old_path).unwrap_or(&file.path);
        let new_file_path = format!("{}{}", new_path, relative);

        let url = format!("https://api.github.com/repos/{}/contents/{}", repo, file.path);
        let get_res = client
            .0
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !get_res.status().is_success() {
            continue;
        }

        let json: serde_json::Value = get_res.json().await?;
        let content = json["content"].as_str().unwrap_or("");
        let sha = json["sha"].as_str().unwrap_or("");

        let create_url = format!("https://api.github.com/repos/{}/contents/{}", repo, new_file_path);
        let create_body = serde_json::json!({
            "message": format!("Move {} to {}", file.path, new_file_path),
            "content": content
        });

        let create_res = client
            .0
            .put(&create_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .json(&create_body)
            .send()
            .await?;

        if !create_res.status().is_success() {
            continue;
        }

        let delete_body = serde_json::json!({
            "message": format!("Delete old {} after move", file.path),
            "sha": sha
        });

        let _ = client
            .0
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .json(&delete_body)
            .send()
            .await;

        moved_count += 1;
    }

    Ok(moved_count)
}

#[tauri::command]
pub async fn download_secure_photo(
    client: State<'_, HttpClient>,
    remote_path: String,
    repo: String,
    token: String,
    keypair_bytes: Vec<u8>,
) -> Result<Vec<u8>, AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, remote_path);

    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Api(format!("Failed to get file info: {}", res.status())));
    }

    let json: serde_json::Value = res.json().await?;
    let download_url = json["download_url"]
        .as_str()
        .ok_or_else(|| AppError::Api("No download URL found".into()))?;

    let content_res = client
        .0
        .get(download_url)
        .header("User-Agent", "vortex-image")
        .send()
        .await?;

    if !content_res.status().is_success() {
        return Err(AppError::Api(format!("Failed to download file: {}", content_res.status())));
    }

    let encrypted_bytes = content_res.bytes().await?;

    let encrypted_data: EncryptedFileData = serde_json::from_slice(&encrypted_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid encrypted file format: {}", e)))?;

    let compressed_bytes = {
        // Deserialize the encrypted payload
        let encrypted_payload: EncryptedPayload = serde_json::from_slice(&encrypted_data.data)
            .map_err(|e| AppError::Validation(format!("Invalid encrypted payload: {}", e)))?;
        
        // Decrypt using keypair bytes
        decrypt_with_keypair_bytes(&encrypted_payload, &keypair_bytes)
            .map_err(|e| AppError::Validation(format!("Decryption failed: {}", e)))?
    };

    let compressed_file: crate::compress::CompressedFileData = serde_json::from_slice(&compressed_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid compressed file format: {}", e)))?;

    let final_image = crate::compress::decompress_file_data(&compressed_file)
        .map_err(|e| AppError::Validation(format!("Decompression failed: {}", e)))?;

    Ok(final_image)
}

#[tauri::command]
pub async fn upload_secure_message(
    client: State<'_, HttpClient>,
    content: String,
    repo: String,
    token: String,
    filename: String,
    public_bundle: PublicBundle,
) -> Result<UploadResult, AppError> {
    validate_repo(&repo)?;
    let safe_filename = sanitize_filename(&filename);

    if safe_filename.is_empty() {
        return Err(AppError::Validation("Invalid filename".into()));
    }

    let encrypted_payload = encrypt(content.as_bytes(), &public_bundle)
        .map_err(|e| AppError::Validation(format!("Encryption failed: {}", e)))?;

    let encrypted_bytes = serde_json::to_vec(&encrypted_payload)
        .map_err(|e| AppError::Validation(format!("Serialization failed: {}", e)))?;

    let encoded = STANDARD.encode(&encrypted_bytes);
    
    let upload_path = format!("messages/{}.msg", safe_filename);
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, upload_path);

    let body = serde_json::json!({
        "message": format!("Upload secure message {}", safe_filename),
        "content": encoded
    });

    let res = client
        .0
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let err = res.text().await.unwrap_or_default();
        return Err(AppError::Api(format!("Upload failed ({}): {}", status, err)));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(UploadResult {
        url: json["content"]["html_url"].as_str().unwrap_or("").to_string(),
        sha: json["content"]["sha"].as_str().unwrap_or("").to_string(),
    })
}

#[tauri::command]
pub async fn download_secure_message(
    client: State<'_, HttpClient>,
    filename: String,
    repo: String,
    token: String,
    keypair_bytes: Vec<u8>,
) -> Result<String, AppError> {
    validate_repo(&repo)?;
    let safe_filename = sanitize_filename(&filename);

    let remote_path = if safe_filename.ends_with(".msg") {
        format!("messages/{}", safe_filename)
    } else {
        format!("messages/{}.msg", safe_filename)
    };

    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, remote_path);

    let res = client
        .0
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::Api(format!("Failed to get message: {}", res.status())));
    }

    let json: serde_json::Value = res.json().await?;

    let content_b64 = json["content"]
        .as_str()
        .ok_or_else(|| AppError::Api("No content found".into()))?
        .replace('\n', "");

    let encrypted_bytes = STANDARD.decode(&content_b64)
        .map_err(|e| AppError::Validation(format!("Base64 decode failed: {}", e)))?;

    let encrypted_payload: EncryptedPayload = serde_json::from_slice(&encrypted_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid encrypted payload: {}", e)))?;

    let decrypted_bytes = decrypt_with_keypair_bytes(&encrypted_payload, &keypair_bytes)
        .map_err(|e| AppError::Validation(format!("Decryption failed: {}", e)))?;

    let message = String::from_utf8(decrypted_bytes)
        .map_err(|e| AppError::Validation(format!("Invalid UTF-8 message: {}", e)))?;

    Ok(message)
}