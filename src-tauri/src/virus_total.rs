use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use sha2::{Sha256, Digest};
use tokio::time::sleep;

// Constants
const VT_API_URL: &str = "https://www.virustotal.com/api/v3";
const CACHE_EXPIRATION: Duration = Duration::from_secs(86400); // 24 hours
const API_RATE_LIMIT: Duration = Duration::from_secs(15); // 15 seconds between API calls for free tier

// Cache for storing scan results to avoid rescanning
static SCAN_CACHE: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, (ScanResult, Instant)>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Last API call timestamp for rate limiting
static LAST_API_CALL: once_cell::sync::Lazy<Arc<Mutex<Option<Instant>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEntry {
    pub detected: bool,
    pub version: Option<String>,
    pub result: Option<String>,
    pub engine_name: String,
    pub engine_version: Option<String>,
    pub engine_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_hash: String,
    pub scan_date: chrono::DateTime<chrono::Utc>,
    pub status: ScanStatus,
    pub detection_count: Option<u32>,
    pub total_engines: Option<u32>,
    pub permalink: Option<String>,
    pub vendor_results: Option<HashMap<String, ScanEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub success: bool,
    pub message: String,
    pub result: Option<ScanResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Clean,
    Suspicious,
    Malicious,
}

pub struct VirusTotal {
    client: Client,
    api_key: String,
}

impl Clone for VirusTotal {
    fn clone(&self) -> Self {
        VirusTotal {
            client: Client::new(),
            api_key: self.api_key.clone(),
        }
    }
}

impl VirusTotal {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        VirusTotal {
            client,
            api_key,
        }
    }

    pub async fn test_api_key(&self) -> Result<bool, String> {
        self.rate_limit().await?;

        let response = self.client.get(&format!("{}/users/current", VT_API_URL))
            .header("x-apikey", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to validate API key: {}", e))?;

        Ok(response.status().is_success())
    }

    pub async fn scan_file<P: AsRef<Path>>(&self, file_path: P) -> Result<ScanResult, String> {
        let path = PathBuf::from(file_path.as_ref());
        println!("Scanning file: {}", path.display());

        // Validate file
        if !path.exists() {
            return Err("File does not exist".to_string());
        }

        // Get file metadata
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown file")
            .to_string();

        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        // Calculate file hash
        println!("Calculating file hash");
        let file_hash = calculate_file_hash(&path).await?;

        // Check cache
        {
            let cache = SCAN_CACHE.lock().unwrap();
            if let Some((cached_result, timestamp)) = cache.get(&file_hash) {
                if timestamp.elapsed() < CACHE_EXPIRATION {
                    println!("Using cached results");
                    return Ok(cached_result.clone());
                }
            }
        }

        // Upload and scan file
        println!("Uploading file to VirusTotal");

        self.rate_limit().await?;

        // Read file into memory for multipart form
        let mut file = std::fs::File::open(&path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let file_name_for_upload = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        
        // Create a form part for the file
        let part = multipart::Part::bytes(buffer)
            .file_name(file_name_for_upload.clone());
        
        // Create the form with the file part
        let form = multipart::Form::new()
            .part("file", part);

        let response = self.client.post(&format!("{}/files", VT_API_URL))
            .header("x-apikey", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to upload file: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Upload failed: {}", response.status()));
        }

        let upload_result = response.json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse upload response: {}", e))?;

        let analysis_id = upload_result["data"]["id"].as_str()
            .ok_or("Missing analysis ID")?;

        // Poll for analysis completion with retry mechanism
        println!("Analyzing file");

        let mut attempts = 0;
        let max_attempts = 30;

        while attempts < max_attempts {
            self.rate_limit().await?;

            let response = self.client.get(&format!("{}/analyses/{}", VT_API_URL, analysis_id))
                .header("x-apikey", &self.api_key)
                .send()
                .await
                .map_err(|e| format!("Analysis failed: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("Analysis failed: {}", response.status()));
            }

            let analysis_result = response.json::<serde_json::Value>()
                .await
                .map_err(|e| format!("Failed to parse analysis: {}", e))?;

            let status = analysis_result["data"]["attributes"]["status"].as_str()
                .ok_or("Missing status")?;

            if status == "completed" {
                let stats = &analysis_result["data"]["attributes"]["stats"];
                let results = &analysis_result["data"]["attributes"]["results"];

                let malicious = stats["malicious"].as_u64().unwrap_or(0);
                let suspicious = stats["suspicious"].as_u64().unwrap_or(0);
                let total = stats["total"].as_u64().unwrap_or(0);

                let mut vendor_results = HashMap::new();
                if let Some(obj) = results.as_object() {
                    for (engine, result) in obj {
                        vendor_results.insert(engine.clone(), ScanEntry {
                            detected: result["category"].as_str().unwrap_or("") == "malicious",
                            version: result["engine_version"].as_str().map(String::from),
                            result: result["result"].as_str().map(String::from),
                            engine_name: engine.clone(),
                            engine_version: result["engine_version"].as_str().map(String::from),
                            engine_update: result["engine_update"].as_str().map(String::from),
                        });
                    }
                }

                let status = if malicious > 0 {
                    ScanStatus::Malicious
                } else if suspicious > 0 {
                    ScanStatus::Suspicious
                } else {
                    ScanStatus::Clean
                };

                let result = ScanResult {
                    file_path: path.to_string_lossy().to_string(),
                    file_name,
                    file_size,
                    file_hash: file_hash.clone(),
                    scan_date: chrono::Utc::now(),
                    status,
                    detection_count: Some((malicious + suspicious) as u32),
                    total_engines: Some(total as u32),
                    permalink: Some(format!("https://www.virustotal.com/gui/file/{}/detection", file_hash)),
                    vendor_results: Some(vendor_results),
                };

                // Cache the result
                {
                    let mut cache = SCAN_CACHE.lock().unwrap();
                    cache.insert(file_hash, (result.clone(), Instant::now()));
                }

                println!("Scan completed successfully");
                return Ok(result);
            }

            let progress = 60 + ((attempts as f32 / max_attempts as f32) * 30.0) as u8;
            println!("Analyzing file: {}%", progress);

            attempts += 1;
            sleep(Duration::from_secs(2)).await;
        }

        Err("Analysis timed out".to_string())
    }

    // Rate limit API calls
    async fn rate_limit(&self) -> Result<(), String> {
        let wait_needed;
        
        {
            let last_call = LAST_API_CALL.lock().unwrap();
            
            if let Some(timestamp) = *last_call {
                let elapsed = timestamp.elapsed();
                if elapsed < API_RATE_LIMIT {
                    wait_needed = Some(API_RATE_LIMIT - elapsed);
                } else {
                    wait_needed = None;
                }
            } else {
                wait_needed = None;
            }
        }
        
        if let Some(wait_time) = wait_needed {
            println!("Rate limiting: waiting for {} seconds", wait_time.as_secs());
            sleep(wait_time).await;
        }
        
        let mut last_call = LAST_API_CALL.lock().unwrap();
        *last_call = Some(Instant::now());
        
        Ok(())
    }
}

async fn calculate_file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// Helper function to emit progress updates
fn emit_progress(app_handle: &AppHandle, step: u8, message: &str, progress: u8) -> Result<(), String> {
    app_handle.emit("scan-progress", (step, message, progress))
        .map_err(|e| format!("Failed to emit progress: {}", e))
}

// Tauri command to scan a file
#[tauri::command]
pub async fn scan_file(
    file_path: String,
    state: tauri::State<'_, crate::AppState>,
    app_handle: AppHandle,
) -> Result<ScanResponse, String> {
    let api_key = state.api_key.lock().await.clone();
    
    match api_key {
        Some(key) => {
            emit_progress(&app_handle, 1, "Starting scan", 10).ok();
            
            let vt = VirusTotal::new(key);
            match vt.scan_file(&file_path).await {
                Ok(result) => {
                    // Add to scan history
                    let mut history = state.scan_history.lock().await;
                    history.push(result.clone());
                    
                    emit_progress(&app_handle, 3, "Scan completed", 100).ok();
                    
                    Ok(ScanResponse {
                        success: true,
                        message: "Scan completed successfully".to_string(),
                        result: Some(result),
                    })
                },
                Err(e) => {
                    emit_progress(&app_handle, 3, "Scan failed", 100).ok();
                    
                    Ok(ScanResponse {
                        success: false,
                        message: format!("Scan failed: {}", e),
                        result: None,
                    })
                }
            }
        },
        None => Err("API key not initialized".to_string()),
    }
}

// Tauri command to test API key
#[tauri::command]
pub async fn test_api_key(api_key: String) -> Result<bool, String> {
    let vt = VirusTotal::new(api_key);
    vt.test_api_key().await
}

// Tauri command to check if setup is complete
#[tauri::command]
pub async fn is_setup_complete(state: tauri::State<'_, crate::AppState>) -> Result<bool, String> {
    Ok(*state.is_setup_complete.lock().await)
}

// Tauri command to get scan history
#[tauri::command]
pub async fn get_scan_history(state: tauri::State<'_, crate::AppState>) -> Result<Vec<ScanResult>, String> {
    let history = state.scan_history.lock().await;
    Ok(history.clone())
}

// Tauri command to clear scan history
#[tauri::command]
pub async fn clear_scan_history(state: tauri::State<'_, crate::AppState>) -> Result<(), String> {
    let mut history = state.scan_history.lock().await;
    history.clear();
    Ok(())
}

// This function was previously used but is now handled directly in the scan_file command
#[allow(dead_code)]
pub async fn scan_file_vt(
    file_path: String,
    api_key: String,
    app_handle: AppHandle,
) -> Result<ScanResponse, String> {
    emit_progress(&app_handle, 1, "Starting scan", 10).ok();
    
    let vt = VirusTotal::new(api_key);
    match vt.scan_file(&file_path).await {
        Ok(result) => {
            emit_progress(&app_handle, 3, "Scan completed", 100).ok();
            
            Ok(ScanResponse {
                success: true,
                message: "Scan completed successfully".to_string(),
                result: Some(result),
            })
        },
        Err(e) => {
            emit_progress(&app_handle, 3, "Scan failed", 100).ok();
            
            Ok(ScanResponse {
                success: false,
                message: format!("Scan failed: {}", e),
                result: None,
            })
        }
    }
}

// Delete file command
#[tauri::command]
pub async fn delete_file(file_path: String) -> Result<bool, String> {
    use std::fs;

    match fs::remove_file(&file_path) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to delete file: {}", e)),
    }
}
