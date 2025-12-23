//! Rust Module - 14 functions, 7 structs
//! Core functionality: Backend operations and data processing
//! External crates: 6 dependencies

use serde::{Deserialize, Serialize};
use crate::compress::{
    Algorithm as CompressAlgorithm, CompressionSettings,
    compress, decompress
};
use crate::crypto::{
    encrypt_with_password, decrypt_with_password,
    encrypt, decrypt, HybridKeypair, PublicBundle, hash_data
};
use crate::github::AppError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PipelineOperation {
    
    Compress {
        algorithm: String,
        level: i32,
    },
    
    EncryptPassword {
        
        #[serde(skip)]
        password: Option<String>,
    },
    
    EncryptHybridPQ {
        
        recipient_bundle: Option<PublicBundle>,
    },
    
    Hash,
    
    Base64Encode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineLayer {
    pub id: String,
    pub operation: PipelineOperation,
    pub enabled: bool,
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layers: Vec<PipelineLayer>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineResult {
    pub data: Vec<u8>,
    pub original_size: usize,
    pub final_size: usize,
    pub layers_applied: Vec<LayerResult>,
    pub checksum: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer_id: String,
    pub operation_type: String,
    pub input_size: usize,
    pub output_size: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineMetadata {
    pub version: u8,
    pub layers: Vec<LayerMetadata>,
    pub original_checksum: Vec<u8>,
    pub original_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub operation_type: String,
    pub params: serde_json::Value,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            id: format!("pipeline-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()),
            name: "Default Pipeline".to_string(),
            description: "Standard compression + encryption".to_string(),
            layers: vec![],
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

pub struct PipelineContext {
    pub passwords: std::collections::HashMap<String, String>,
    pub keypair: Option<HybridKeypair>,
}

impl Default for PipelineContext {
    fn default() -> Self {
        Self {
            passwords: std::collections::HashMap::new(),
            keypair: None,
        }
    }
}

pub fn process_pipeline(
    data: &[u8],
    config: &PipelineConfig,
    context: &PipelineContext,
) -> Result<PipelineResult, PipelineError> {
    let original_size = data.len();
    let original_checksum = hash_data(data).to_vec();
    
    let mut current_data = data.to_vec();
    let mut layers_applied = Vec::new();
    let mut layer_metadata = Vec::new();

    let mut sorted_layers: Vec<_> = config.layers.iter()
        .filter(|l| l.enabled)
        .collect();
    sorted_layers.sort_by_key(|l| l.order);
    
    for layer in sorted_layers {
        let input_size = current_data.len();
        
        let result = apply_layer(&current_data, layer, context);
        
        match result {
            Ok((output, metadata)) => {
                layers_applied.push(LayerResult {
                    layer_id: layer.id.clone(),
                    operation_type: get_operation_type(&layer.operation),
                    input_size,
                    output_size: output.len(),
                    success: true,
                    error: None,
                });
                layer_metadata.push(metadata);
                current_data = output;
            }
            Err(e) => {
                layers_applied.push(LayerResult {
                    layer_id: layer.id.clone(),
                    operation_type: get_operation_type(&layer.operation),
                    input_size,
                    output_size: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
                return Err(e);
            }
        }
    }

    let metadata = PipelineMetadata {
        version: 1,
        layers: layer_metadata,
        original_checksum: original_checksum.clone(),
        original_size,
    };
    
    let metadata_json = serde_json::to_vec(&metadata)
        .map_err(|e| PipelineError::Serialization(e.to_string()))?;

    let mut final_data = Vec::with_capacity(4 + metadata_json.len() + current_data.len());
    final_data.extend_from_slice(&(metadata_json.len() as u32).to_le_bytes());
    final_data.extend_from_slice(&metadata_json);
    final_data.extend_from_slice(&current_data);
    
    let final_size = final_data.len();
    let final_checksum = hash_data(&final_data).to_vec();
    
    Ok(PipelineResult {
        data: final_data,
        original_size,
        final_size,
        layers_applied,
        checksum: final_checksum,
    })
}

pub fn reverse_pipeline(
    data: &[u8],
    context: &PipelineContext,
) -> Result<PipelineResult, PipelineError> {
    if data.len() < 4 {
        return Err(PipelineError::InvalidData("Data too short".into()));
    }

    let metadata_len = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;
    if data.len() < 4 + metadata_len {
        return Err(PipelineError::InvalidData("Invalid metadata length".into()));
    }
    
    let metadata: PipelineMetadata = serde_json::from_slice(&data[4..4 + metadata_len])
        .map_err(|e| PipelineError::Serialization(e.to_string()))?;
    
    let mut current_data = data[4 + metadata_len..].to_vec();
    let mut layers_applied = Vec::new();

    for layer_meta in metadata.layers.iter().rev() {
        let input_size = current_data.len();
        
        let result = reverse_layer(&current_data, layer_meta, context);
        
        match result {
            Ok(output) => {
                layers_applied.push(LayerResult {
                    layer_id: format!("reverse-{}", layer_meta.operation_type),
                    operation_type: format!("reverse_{}", layer_meta.operation_type),
                    input_size,
                    output_size: output.len(),
                    success: true,
                    error: None,
                });
                current_data = output;
            }
            Err(e) => {
                layers_applied.push(LayerResult {
                    layer_id: format!("reverse-{}", layer_meta.operation_type),
                    operation_type: format!("reverse_{}", layer_meta.operation_type),
                    input_size,
                    output_size: 0,
                    success: false,
                    error: Some(e.to_string()),
                });
                return Err(e);
            }
        }
    }

    let final_checksum = hash_data(&current_data).to_vec();
    if final_checksum != metadata.original_checksum {
        return Err(PipelineError::ChecksumMismatch);
    }
    
    Ok(PipelineResult {
        data: current_data,
        original_size: metadata.original_size,
        final_size: metadata.original_size,
        layers_applied,
        checksum: final_checksum,
    })
}

fn apply_layer(
    data: &[u8],
    layer: &PipelineLayer,
    context: &PipelineContext,
) -> Result<(Vec<u8>, LayerMetadata), PipelineError> {
    match &layer.operation {
        PipelineOperation::Compress { algorithm, level } => {
            let settings = CompressionSettings {
                algorithm: CompressAlgorithm::from(algorithm.as_str()),
                level: *level,
                prefer_speed: false,
            };
            let result = compress(data, &settings)
                .map_err(|e| PipelineError::Compression(e.to_string()))?;
            
            Ok((result.data, LayerMetadata {
                operation_type: "compress".to_string(),
                params: serde_json::json!({
                    "algorithm": algorithm,
                    "level": level
                }),
            }))
        }
        
        PipelineOperation::EncryptPassword { .. } => {
            let password = context.passwords.get(&layer.id)
                .ok_or_else(|| PipelineError::MissingPassword(layer.id.clone()))?;
            
            let encrypted = encrypt_with_password(data, password.as_bytes())
                .map_err(|e| PipelineError::Encryption(e.to_string()))?;
            
            Ok((encrypted, LayerMetadata {
                operation_type: "encrypt_password".to_string(),
                params: serde_json::json!({}),
            }))
        }
        
        PipelineOperation::EncryptHybridPQ { recipient_bundle } => {
            let bundle = recipient_bundle.as_ref()
                .ok_or_else(|| PipelineError::MissingRecipient)?;
            
            let payload = encrypt(data, bundle)
                .map_err(|e| PipelineError::Encryption(e.to_string()))?;
            
            let serialized = serde_json::to_vec(&payload)
                .map_err(|e| PipelineError::Serialization(e.to_string()))?;
            
            Ok((serialized, LayerMetadata {
                operation_type: "encrypt_hybrid_pq".to_string(),
                params: serde_json::json!({}),
            }))
        }
        
        PipelineOperation::Hash => {
            
            let hash = hash_data(data);
            Ok((data.to_vec(), LayerMetadata {
                operation_type: "hash".to_string(),
                params: serde_json::json!({
                    "hash": hex::encode(hash)
                }),
            }))
        }
        
        PipelineOperation::Base64Encode => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let encoded = STANDARD.encode(data);
            Ok((encoded.into_bytes(), LayerMetadata {
                operation_type: "base64_encode".to_string(),
                params: serde_json::json!({}),
            }))
        }
    }
}

fn reverse_layer(
    data: &[u8],
    metadata: &LayerMetadata,
    context: &PipelineContext,
) -> Result<Vec<u8>, PipelineError> {
    match metadata.operation_type.as_str() {
        "compress" => {
            let algorithm = metadata.params["algorithm"].as_str().unwrap_or("zstd");
            decompress(data, CompressAlgorithm::from(algorithm))
                .map_err(|e| PipelineError::Compression(e.to_string()))
        }
        
        "encrypt_password" => {
            
            for password in context.passwords.values() {
                if let Ok(decrypted) = decrypt_with_password(data, password.as_bytes()) {
                    return Ok(decrypted);
                }
            }
            Err(PipelineError::MissingPassword("No valid password found".into()))
        }
        
        "encrypt_hybrid_pq" => {
            let keypair = context.keypair.as_ref()
                .ok_or_else(|| PipelineError::MissingKeypair)?;
            
            let payload = serde_json::from_slice(data)
                .map_err(|e| PipelineError::Serialization(e.to_string()))?;
            
            decrypt(&payload, keypair)
                .map_err(|e| PipelineError::Encryption(e.to_string()))
        }
        
        "hash" => {
            
            Ok(data.to_vec())
        }
        
        "base64_encode" => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let decoded = STANDARD.decode(data)
                .map_err(|e| PipelineError::Encoding(e.to_string()))?;
            Ok(decoded)
        }
        
        _ => Err(PipelineError::UnknownOperation(metadata.operation_type.clone()))
    }
}

fn get_operation_type(op: &PipelineOperation) -> String {
    match op {
        PipelineOperation::Compress { .. } => "compress".to_string(),
        PipelineOperation::EncryptPassword { .. } => "encrypt_password".to_string(),
        PipelineOperation::EncryptHybridPQ { .. } => "encrypt_hybrid_pq".to_string(),
        PipelineOperation::Hash => "hash".to_string(),
        PipelineOperation::Base64Encode => "base64_encode".to_string(),
    }
}

#[derive(Debug)]
pub enum PipelineError {
    Compression(String),
    Encryption(String),
    Serialization(String),
    Encoding(String),
    InvalidData(String),
    MissingPassword(String),
    MissingKeypair,
    MissingRecipient,
    ChecksumMismatch,
    UnknownOperation(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compression(e) => write!(f, "Compression error: {}", e),
            Self::Encryption(e) => write!(f, "Encryption error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Encoding(e) => write!(f, "Encoding error: {}", e),
            Self::InvalidData(e) => write!(f, "Invalid data: {}", e),
            Self::MissingPassword(id) => write!(f, "Missing password for layer: {}", id),
            Self::MissingKeypair => write!(f, "Keypair required but not provided"),
            Self::MissingRecipient => write!(f, "Recipient bundle required"),
            Self::ChecksumMismatch => write!(f, "Checksum verification failed"),
            Self::UnknownOperation(op) => write!(f, "Unknown operation: {}", op),
        }
    }
}

impl std::error::Error for PipelineError {}

pub fn get_preset_pipelines() -> Vec<PipelineConfig> {
    vec![
        
        PipelineConfig {
            id: "preset-fast-compress".to_string(),
            name: "Fast Compression".to_string(),
            description: "LZ4 compression for speed".to_string(),
            layers: vec![
                PipelineLayer {
                    id: "lz4-compress".to_string(),
                    operation: PipelineOperation::Compress {
                        algorithm: "lz4".to_string(),
                        level: 1,
                    },
                    enabled: true,
                    order: 0,
                },
            ],
            created_at: 0,
            updated_at: 0,
        },
        
        PipelineConfig {
            id: "preset-max-compress".to_string(),
            name: "Maximum Compression".to_string(),
            description: "Zstd level 19 for best ratio".to_string(),
            layers: vec![
                PipelineLayer {
                    id: "zstd-max".to_string(),
                    operation: PipelineOperation::Compress {
                        algorithm: "zstd".to_string(),
                        level: 19,
                    },
                    enabled: true,
                    order: 0,
                },
            ],
            created_at: 0,
            updated_at: 0,
        },
        
        PipelineConfig {
            id: "preset-password-encrypt".to_string(),
            name: "Password Protected".to_string(),
            description: "Compress + password encryption".to_string(),
            layers: vec![
                PipelineLayer {
                    id: "zstd-compress".to_string(),
                    operation: PipelineOperation::Compress {
                        algorithm: "zstd".to_string(),
                        level: 3,
                    },
                    enabled: true,
                    order: 0,
                },
                PipelineLayer {
                    id: "password-encrypt".to_string(),
                    operation: PipelineOperation::EncryptPassword { password: None },
                    enabled: true,
                    order: 1,
                },
            ],
            created_at: 0,
            updated_at: 0,
        },
        
        PipelineConfig {
            id: "preset-pq-secure".to_string(),
            name: "Post-Quantum Secure".to_string(),
            description: "ML-KEM-1024 + X25519 hybrid encryption".to_string(),
            layers: vec![
                PipelineLayer {
                    id: "zstd-compress".to_string(),
                    operation: PipelineOperation::Compress {
                        algorithm: "zstd".to_string(),
                        level: 3,
                    },
                    enabled: true,
                    order: 0,
                },
                PipelineLayer {
                    id: "pq-encrypt".to_string(),
                    operation: PipelineOperation::EncryptHybridPQ { recipient_bundle: None },
                    enabled: true,
                    order: 1,
                },
            ],
            created_at: 0,
            updated_at: 0,
        },
        
        PipelineConfig {
            id: "preset-max-security".to_string(),
            name: "Maximum Security".to_string(),
            description: "Triple layer: compress + password + PQ encryption".to_string(),
            layers: vec![
                PipelineLayer {
                    id: "zstd-compress".to_string(),
                    operation: PipelineOperation::Compress {
                        algorithm: "zstd".to_string(),
                        level: 6,
                    },
                    enabled: true,
                    order: 0,
                },
                PipelineLayer {
                    id: "password-layer".to_string(),
                    operation: PipelineOperation::EncryptPassword { password: None },
                    enabled: true,
                    order: 1,
                },
                PipelineLayer {
                    id: "pq-layer".to_string(),
                    operation: PipelineOperation::EncryptHybridPQ { recipient_bundle: None },
                    enabled: true,
                    order: 2,
                },
                PipelineLayer {
                    id: "base64-layer".to_string(),
                    operation: PipelineOperation::Base64Encode,
                    enabled: true,
                    order: 3,
                },
            ],
            created_at: 0,
            updated_at: 0,
        },
    ]
}

#[tauri::command]
pub async fn pipeline_process(
    data: Vec<u8>,
    config: PipelineConfig,
    passwords: std::collections::HashMap<String, String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<PipelineResult, AppError> {
    let keypair = if let Some(bytes) = keypair_bytes {
        Some(HybridKeypair::from_bytes(&bytes)
            .map_err(|e| AppError::Validation(e.to_string()))?)
    } else {
        None
    };
    
    let context = PipelineContext { passwords, keypair };
    
    process_pipeline(&data, &config, &context)
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub async fn pipeline_reverse(
    data: Vec<u8>,
    passwords: std::collections::HashMap<String, String>,
    keypair_bytes: Option<Vec<u8>>,
) -> Result<PipelineResult, AppError> {
    let keypair = if let Some(bytes) = keypair_bytes {
        Some(HybridKeypair::from_bytes(&bytes)
            .map_err(|e| AppError::Validation(e.to_string()))?)
    } else {
        None
    };
    
    let context = PipelineContext { passwords, keypair };
    
    reverse_pipeline(&data, &context)
        .map_err(|e| AppError::Validation(e.to_string()))
}

#[tauri::command]
pub fn pipeline_get_presets() -> Vec<PipelineConfig> {
    get_preset_pipelines()
}

#[tauri::command]
pub fn pipeline_validate(config: PipelineConfig) -> Result<bool, AppError> {
    
    let mut ids = std::collections::HashSet::new();
    for layer in &config.layers {
        if !ids.insert(&layer.id) {
            return Err(AppError::Validation(format!("Duplicate layer ID: {}", layer.id)));
        }
    }

    for layer in &config.layers {
        match &layer.operation {
            PipelineOperation::Compress { algorithm, level } => {
                let _ = CompressAlgorithm::from(algorithm.as_str());
                if *level < 0 || *level > 22 {
                    return Err(AppError::Validation(format!(
                        "Invalid compression level: {} (must be 0-22)", level
                    )));
                }
            }
            _ => {}
        }
    }
    
    Ok(true)
}

#[tauri::command]
pub fn pipeline_estimate(
    original_size: usize,
    config: PipelineConfig,
) -> serde_json::Value {
    let mut estimated_size = original_size as f64;
    let mut operations = Vec::new();
    
    for layer in config.layers.iter().filter(|l| l.enabled) {
        let (ratio, op_name) = match &layer.operation {
            PipelineOperation::Compress { algorithm, .. } => {
                let ratio = match algorithm.as_str() {
                    "zstd" => 0.4,
                    "lz4" => 0.6,
                    "snap" => 0.65,
                    "brotli" => 0.35,
                    "gzip" => 0.45,
                    _ => 1.0,
                };
                (ratio, format!("Compress ({})", algorithm))
            }
            PipelineOperation::EncryptPassword { .. } => {
                (1.05, "Password Encryption".to_string()) 
            }
            PipelineOperation::EncryptHybridPQ { .. } => {
                (1.1, "PQ Encryption".to_string()) 
            }
            PipelineOperation::Hash => {
                (1.0, "Hash".to_string())
            }
            PipelineOperation::Base64Encode => {
                (1.33, "Base64 Encode".to_string()) 
            }
        };
        
        estimated_size *= ratio;
        operations.push(serde_json::json!({
            "operation": op_name,
            "ratio": ratio,
            "estimated_size_after": estimated_size as usize
        }));
    }
    
    serde_json::json!({
        "original_size": original_size,
        "estimated_final_size": estimated_size as usize,
        "overall_ratio": estimated_size / original_size as f64,
        "operations": operations
    })
}