use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use tauri::Window;
use crate::config::ScannerConfig;

pub struct BackgroundScanner {
    config: Arc<Mutex<ScannerConfig>>,
    window: Arc<Window>,
    scanning: Arc<Mutex<bool>>,
    scan_queue: Arc<Mutex<Vec<std::path::PathBuf>>>,
}

impl BackgroundScanner {
    pub fn new(window: Window) -> Self {
        Self {
            config: Arc::new(Mutex::new(ScannerConfig::default())),
            window: Arc::new(window),
            scanning: Arc::new(Mutex::new(false)),
            scan_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn set_config(&self, config: ScannerConfig) {
        let mut current_config = self.config.lock().await;
        *current_config = config;
    }

    pub async fn start_scanning(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut scanning = self.scanning.lock().await;
        if *scanning {
            return Ok(());
        }
        *scanning = true;

        let scanner = self.clone();
        tokio::spawn(async move {
            scanner.scanning_loop().await;
        });

        Ok(())
    }

    async fn scanning_loop(&self) {
        while *self.scanning.lock().await {
            let config = self.config.lock().await;
            
            // Process batch of files
            self.process_batch(&config).await;
            
            // Wait for next scan interval
            tokio::time::sleep(config.scan_interval).await;
        }
    }

    async fn process_batch(&self, config: &ScannerConfig) {
        let mut queue = self.scan_queue.lock().await;
        let batch: Vec<_> = queue.drain(..config.batch_size.min(queue.len())).collect();
        
        for path in batch {
            let result = self.scan_file(&path, config).await;
            match result {
                Ok(scan_result) => {
                    self.window.emit("scan-complete", json!({
                        "path": path.to_string_lossy(),
                        "result": scan_result
                    })).ok();
                }
                Err(e) => {
                    self.window.emit("scan-error", json!({
                        "path": path.to_string_lossy(),
                        "error": e.to_string()
                    })).ok();
                }
            }
        }
    }

    async fn scan_file(&self, path: &std::path::PathBuf, config: &ScannerConfig) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut attempts = 0;
        while attempts < config.retry_attempts {
            match self.perform_single_scan(path).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts < config.retry_attempts {
                        tokio::time::sleep(config.retry_delay).await;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Err("Max retry attempts exceeded".into())
    }

    async fn perform_single_scan(&self, path: &std::path::PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // TODO: Implement actual virus scanning logic here
        // For now, return a mock result
        Ok(json!({
            "status": "clean",
            "scanned_at": chrono::Utc::now().to_rfc3339(),
            "threats_found": 0
        }))
    }

    pub async fn stop_scanning(&self) {
        let mut scanning = self.scanning.lock().await;
        *scanning = false;
    }

    pub async fn add_to_queue(&self, path: std::path::PathBuf) {
        let mut queue = self.scan_queue.lock().await;
        if !queue.contains(&path) {
            queue.push(path);
        }
    }
}
