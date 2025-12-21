use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::fs;

const CLIENT_ID: &str = "Ov23lijNSMM1i93CQdfQ";

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

    if let Some(err) = token_res.error {
        if err == "authorization_pending" || err == "slow_down" {
            return Ok(None);
        }
        return Err(AppError::Api(err));
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

#[tauri::command]
pub async fn list_photos(
    client: State<'_, HttpClient>,
    repo: String,
    token: String,
) -> Result<Vec<String>, AppError> {
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
        .filter_map(|f| f["download_url"].as_str().map(String::from))
        .collect())
}
