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

const CLIENT_ID: &str = "Ov23lijNSMM1i93CQdfQ";
const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 1000;

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

// Shared HTTP client with connection pooling
pub struct HttpClient(pub Arc<Client>);

impl HttpClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self(Arc::new(client))
    }
}

/// Retry helper with exponential backoff
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
                // Exponential backoff with jitter
                let jitter = rand::random::<u64>() % (delay / 2);
                sleep(Duration::from_millis(delay + jitter)).await;
                delay *= 2;
            }
        }
    }
}

/// Check if error is retryable (rate limit, server error, network)
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
        || status == reqwest::StatusCode::REQUEST_TIMEOUT
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
pub async fn start_oauth(client: State<'_, HttpClient>) -> Result<DeviceCodeResponse, AppError> {
    let res = client
        .0
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", "repo")])
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
) -> Result<Option<String>, AppError> {
    let res = client
        .0
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", CLIENT_ID),
            ("device_code", &device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await?;

    let token_res: TokenResponse = res.json().await?;

    // Check for pending/slow_down first - return None to keep polling
    if let Some(ref err) = token_res.error {
        if err == "authorization_pending" || err == "slow_down" {
            return Ok(None);
        }
        return Err(AppError::Api(err.clone()));
    }

    // Return token if present
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

#[tauri::command]
pub async fn upload_photo(
    app: AppHandle,
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
    filename: String,
    upload_id: String,
) -> Result<UploadResult, AppError> {
    validate_repo(&repo)?;
    let safe_filename = sanitize_filename(&filename);

    if safe_filename.is_empty() {
        return Err(AppError::Validation("Invalid filename".into()));
    }

    // Async file read - doesn't block tokio runtime
    let content = fs::read(&path).await?;
    let total_bytes = content.len() as u64;

    // Emit initial progress
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: 0,
        total_bytes,
        percent: 0,
    });

    if total_bytes > 50 * 1024 * 1024 {
        return upload_lfs_internal(&app, &client.0, content, &repo, &token, &safe_filename, &upload_id).await;
    }

    // Emit encoding progress (50%)
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: total_bytes / 2,
        total_bytes,
        percent: 50,
    });

    let encoded = STANDARD.encode(&content);
    drop(content);

    let upload_path = format!("photos/{}", safe_filename);
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, upload_path);

    let body = serde_json::json!({
        "message": format!("Upload {}", safe_filename),
        "content": encoded
    });

    let res = client
        .0
        .put(&url)
        .timeout(std::time::Duration::from_secs(120))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    // Emit complete
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: total_bytes,
        total_bytes,
        percent: 100,
    });

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

/// Upload photo with optional image processing (EXIF stripping, compression)
#[tauri::command]
pub async fn upload_photo_processed(
    app: AppHandle,
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
    filename: String,
    upload_id: String,
    strip_exif: bool,
    compress: bool,
    quality: Option<u8>,
) -> Result<UploadResult, AppError> {
    validate_repo(&repo)?;
    let safe_filename = sanitize_filename(&filename);

    if safe_filename.is_empty() {
        return Err(AppError::Validation("Invalid filename".into()));
    }

    // Read file
    let content = fs::read(&path).await?;
    let total_bytes = content.len() as u64;

    // Emit initial progress
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: 0,
        total_bytes,
        percent: 0,
    });

    // Process image if needed
    let processed_content = if strip_exif || compress {
        process_image(&content, strip_exif, compress, quality.unwrap_or(85))?
    } else {
        content
    };

    let processed_bytes = processed_content.len() as u64;

    // Emit processing complete (30%)
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: 0,
        total_bytes: processed_bytes,
        percent: 30,
    });

    if processed_bytes > 50 * 1024 * 1024 {
        return upload_lfs_internal(&app, &client.0, processed_content, &repo, &token, &safe_filename, &upload_id).await;
    }

    // Emit encoding progress (50%)
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: processed_bytes / 2,
        total_bytes: processed_bytes,
        percent: 50,
    });

    let encoded = STANDARD.encode(&processed_content);
    drop(processed_content);

    let upload_path = format!("photos/{}", safe_filename);
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, upload_path);

    let body = serde_json::json!({
        "message": format!("Upload {}", safe_filename),
        "content": encoded
    });

    let res = client
        .0
        .put(&url)
        .timeout(std::time::Duration::from_secs(120))
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "vortex-image")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?;

    // Emit complete
    let _ = app.emit("upload-progress", UploadProgress {
        id: upload_id.clone(),
        bytes_sent: processed_bytes,
        total_bytes: processed_bytes,
        percent: 100,
    });

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

/// Process image: strip EXIF and/or compress
fn process_image(content: &[u8], strip_exif: bool, compress: bool, quality: u8) -> Result<Vec<u8>, AppError> {
    // Try to decode the image
    let img = image::load_from_memory(content)
        .map_err(|e| AppError::Validation(format!("Failed to decode image: {}", e)))?;

    // Determine output format based on input
    let format = image::guess_format(content)
        .unwrap_or(ImageFormat::Jpeg);

    let mut output = Cursor::new(Vec::new());

    if compress || strip_exif {
        // Re-encoding strips EXIF data and allows compression
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
                // For unsupported formats, convert to JPEG
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
                img.write_with_encoder(encoder)
                    .map_err(|e| AppError::Validation(format!("Failed to encode image: {}", e)))?;
            }
        }
        Ok(output.into_inner())
    } else {
        // Return original content if no processing needed
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
        .timeout(std::time::Duration::from_secs(300))
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

// ============================================================================
// Repository Management
// ============================================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub description: Option<String>,
    pub html_url: String,
    pub default_branch: String,
}

/// Validates repository name format
/// Valid: alphanumeric, hyphens, underscores, dots
/// Length: 1-100 characters
pub fn validate_repo_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::Validation(
            "Repository name must be 1-100 characters".into(),
        ));
    }

    // Check for valid characters
    let valid = name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.');

    if !valid {
        return Err(AppError::Validation(
            "Repository name can only contain letters, numbers, hyphens, underscores, and dots"
                .into(),
        ));
    }

    // Cannot start or end with dot or hyphen
    if name.starts_with('.') || name.starts_with('-') || name.ends_with('.') || name.ends_with('-')
    {
        return Err(AppError::Validation(
            "Repository name cannot start or end with a dot or hyphen".into(),
        ));
    }

    // Cannot contain consecutive dots
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
        let err = res.text().await.unwrap_or_default();
        return Err(AppError::Api(format!(
            "Failed to create repository ({}): {}",
            status, err
        )));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(RepoInfo {
        name: json["name"].as_str().unwrap_or("").to_string(),
        full_name: json["full_name"].as_str().unwrap_or("").to_string(),
        private: json["private"].as_bool().unwrap_or(false),
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().unwrap_or("").to_string(),
        default_branch: json["default_branch"].as_str().unwrap_or("main").to_string(),
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
        name: json["name"].as_str().unwrap_or("").to_string(),
        full_name: json["full_name"].as_str().unwrap_or("").to_string(),
        private: json["private"].as_bool().unwrap_or(false),
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().unwrap_or("").to_string(),
        default_branch: json["default_branch"].as_str().unwrap_or("main").to_string(),
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
        let err = res.text().await.unwrap_or_default();
        return Err(AppError::Api(format!(
            "Failed to update repository visibility ({}): {}",
            status, err
        )));
    }

    let json: serde_json::Value = res.json().await?;

    Ok(RepoInfo {
        name: json["name"].as_str().unwrap_or("").to_string(),
        full_name: json["full_name"].as_str().unwrap_or("").to_string(),
        private: json["private"].as_bool().unwrap_or(false),
        description: json["description"].as_str().map(|s| s.to_string()),
        html_url: json["html_url"].as_str().unwrap_or("").to_string(),
        default_branch: json["default_branch"].as_str().unwrap_or("main").to_string(),
    })
}

#[tauri::command]
pub async fn list_photos(
    client: State<'_, HttpClient>,
    repo: String,
    token: String,
) -> Result<Vec<PhotoItem>, AppError> {
    validate_repo(&repo)?;

    let url = format!("https://api.github.com/repos/{}/contents/photos", repo);

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

// ============================================================================
// Folder Scanning and Album Upload
// ============================================================================

/// Supported image extensions
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

/// Recursively scans a folder for images
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
            // Skip hidden folders
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

/// Collects all images from a folder (non-recursive, just this folder)
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

/// Collects all images from a folder recursively, flattening the structure
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
            // Skip hidden folders
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

/// Uploads a folder as an album, preserving structure
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

    // Collect images based on mode
    let images = if create_subalbums {
        collect_images_recursive(folder_path, folder_path).await?
    } else {
        collect_images_in_folder(folder_path).await?
    };

    let total_files = images.len();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, image) in images.iter().enumerate() {
        // Emit progress
        let _ = app.emit(
            "batch-upload-progress",
            UploadBatchProgress {
                total_files,
                completed_files: index,
                current_file: image.name.clone(),
                percent: ((index * 100) / total_files.max(1)) as u8,
            },
        );

        // Determine upload path
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

    // Emit completion
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

/// Uploads all images from a folder recursively to the root photos folder
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

    // Collect all images recursively
    let images = collect_images_recursive(folder_path, folder_path).await?;

    let total_files = images.len();
    let mut succeeded = Vec::new();
    let mut failed = Vec::new();

    for (index, image) in images.iter().enumerate() {
        // Emit progress
        let _ = app.emit(
            "batch-upload-progress",
            UploadBatchProgress {
                total_files,
                completed_files: index,
                current_file: image.name.clone(),
                percent: ((index * 100) / total_files.max(1)) as u8,
            },
        );

        // Upload to root photos folder (flatten structure)
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

    // Emit completion
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

/// Helper to upload a single file to GitHub with retry logic
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

    // Retry with exponential backoff
    let result = retry_with_backoff(
        || async {
            let res = client
                .put(&url)
                .timeout(std::time::Duration::from_secs(120))
                .header("Authorization", format!("Bearer {}", token))
                .header("User-Agent", "vortex-image")
                .header("Accept", "application/vnd.github+json")
                .json(&body)
                .send()
                .await?;

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

// ============================================================================
// Album Listing
// ============================================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct Album {
    pub name: String,
    pub path: String,
    pub photo_count: usize,
    pub children: Vec<Album>,
}

/// Lists all albums (folders) in the photos directory
#[tauri::command]
pub async fn list_albums(
    client: State<'_, HttpClient>,
    repo: String,
    token: String,
) -> Result<Vec<Album>, AppError> {
    validate_repo(&repo)?;

    // Get the contents of the photos folder
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

            // Recursively get album contents
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
            // Check if it's an image
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

// ============================================================================
// Photo Download and Delete Operations
// ============================================================================

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub id: String,
    pub bytes_received: u64,
    pub total_bytes: u64,
    pub percent: u8,
}

/// Downloads a photo from GitHub to local storage
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

    // Emit initial progress
    let _ = app.emit("download-progress", DownloadProgress {
        id: download_id.clone(),
        bytes_received: 0,
        total_bytes: 0,
        percent: 0,
    });

    // Get file info from GitHub
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

    // Download the file content
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

    // Emit progress
    let _ = app.emit("download-progress", DownloadProgress {
        id: download_id.clone(),
        bytes_received: content.len() as u64,
        total_bytes,
        percent: 100,
    });

    // Determine local path
    let filename = remote_path.split('/').last().unwrap_or("photo");
    let local_path = if let Some(dir) = local_dir {
        std::path::Path::new(&dir).join(filename)
    } else {
        // Use system downloads folder
        let downloads = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        downloads.join(filename)
    };

    // Write to file
    fs::write(&local_path, &content).await?;

    Ok(local_path.to_string_lossy().to_string())
}

/// Deletes a photo from GitHub repository
#[tauri::command]
pub async fn delete_photo(
    client: State<'_, HttpClient>,
    path: String,
    repo: String,
    token: String,
) -> Result<(), AppError> {
    validate_repo(&repo)?;

    // First, get the file's SHA (required for deletion)
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

    // Delete the file
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
        let err = delete_res.text().await.unwrap_or_default();
        return Err(AppError::Api(format!("Failed to delete file ({}): {}", status, err)));
    }

    Ok(())
}

/// Removes a local file
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

/// Gets detailed info about a local image file
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

// ============================================================================
// Album Management
// ============================================================================

/// Deletes an entire album (folder) from GitHub repository
#[tauri::command]
pub async fn delete_album(
    client: State<'_, HttpClient>,
    album_path: String,
    repo: String,
    token: String,
) -> Result<u32, AppError> {
    validate_repo(&repo)?;

    // Get all files in the album recursively
    let files = get_album_files_recursive(&client.0, &repo, &token, &album_path).await?;
    
    if files.is_empty() {
        return Err(AppError::Validation("Album is empty or does not exist".into()));
    }

    let mut deleted_count = 0u32;

    // Delete each file
    for file in files {
        let url = format!("https://api.github.com/repos/{}/contents/{}", repo, file.path);
        
        // Get file SHA
        let get_res = client
            .0
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "vortex-image")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        if !get_res.status().is_success() {
            continue; // Skip files that don't exist
        }

        let json: serde_json::Value = get_res.json().await?;
        let sha = match json["sha"].as_str() {
            Some(s) => s,
            None => continue,
        };

        // Delete the file
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

/// Renames an album by moving all files to a new path
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

    // Get parent path
    let parent = std::path::Path::new(&old_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let new_path = if parent.is_empty() {
        safe_new_name.clone()
    } else {
        format!("{}/{}", parent, safe_new_name)
    };

    // Get all files in the album
    let files = get_album_files_recursive(&client.0, &repo, &token, &old_path).await?;
    
    if files.is_empty() {
        return Err(AppError::Validation("Album is empty or does not exist".into()));
    }

    let mut moved_count = 0u32;

    for file in files {
        // Calculate new path
        let relative = file.path.strip_prefix(&old_path).unwrap_or(&file.path);
        let new_file_path = format!("{}{}", new_path, relative);

        // Get file content
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

        // Create file at new location
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

        // Delete old file
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
