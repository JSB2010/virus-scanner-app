use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{Duration, sleep};
use futures::stream::{self, StreamExt};
use crate::settings::Settings;
use crate::virus_total::scan_file_internal;
use crate::{AppState, ScanResult, ScanStatus};
use crate::scanner_config::ScannerConfig;

pub struct BackgroundScanner {
    state: Arc<AppState>,
    config: Arc<Mutex<ScannerConfig>>,
    scan_semaphore: Arc<Semaphore>,
}

impl BackgroundScanner {
    pub fn new(state: Arc<AppState>) -> Self {
        let config = Arc::new(Mutex::new(ScannerConfig::default()));
        let scan_semaphore = Arc::new(Semaphore::new(config.blocking_lock().max_concurrent_scans));
        Self { 
            state,
            config,
            scan_semaphore,
        }
    }

    async fn scan_single_file(&self, file_path: String, retry_count: u32) -> Result<ScanResult, String> {
        let _permit = self.scan_semaphore.acquire().await.map_err(|e| e.to_string())?;
        
        let result = scan_file_internal(&file_path, &self.state).await;
        
        if let Err(ref e) = result {
            let config = self.config.lock().await;
            if retry_count < config.retry_attempts {
                sleep(Duration::from_secs(config.retry_delay_seconds)).await;
                return self.scan_single_file(file_path, retry_count + 1).await;
            }
        }
        
        result
    }

    pub async fn start(&self) {
        let state = self.state.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            loop {
                // Load current settings
                let settings = match Settings::load() {
                    Ok(s) => s,
                    Err(_) => {
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                // Check if auto-rescan is enabled and get interval
                if let Some(interval_hours) = settings.auto_rescan_interval {
                    let config = config.lock().await;
                    let batch_size = config.scan_batch_size;
                    
                    // Get files to scan
                    let files_to_scan = {
                        let mut files = Vec::new();
                        let quarantined_files = state.quarantined_files.lock().await;
                        let scan_history = state.scan_history.lock().await;
                        
                        for file in quarantined_files.iter() {
                            if let Ok(metadata) = tokio::fs::metadata(&file.path).await {
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(elapsed) = modified.elapsed() {
                                        // Check if file needs rescanning
                                        let last_scan = scan_history.iter()
                                            .filter(|scan| scan.file_path == file.path)
                                            .max_by_key(|scan| scan.scan_date);
                                            
                                        let needs_rescan = match last_scan {
                                            Some(scan) => {
                                                let scan_age = scan.scan_date.elapsed().unwrap_or_default();
                                                scan_age.as_secs() > interval_hours * 3600
                                            }
                                            None => true
                                        };
                                        
                                        if needs_rescan {
                                            files.push(file.path.clone());
                                        }
                                    }
                                }
                            }
                        }
                        files
                    };

                    // Process files in batches
                    for batch in files_to_scan.chunks(batch_size) {
                        let futures = stream::iter(batch)
                            .map(|file_path| {
                                let scanner = self.clone();
                                async move {
                                    let result = scanner.scan_single_file(file_path.to_string(), 0).await;
                                    (file_path, result)
                                }
                            })
                            .buffer_unordered(config.max_concurrent_scans);

                        // Process results
                        futures.for_each(|(file_path, result)| async {
                            match result {
                                Ok(scan_result) => {
                                    let mut history = state.scan_history.lock().await;
                                    if history.len() >= settings.scan_history_limit as usize {
                                        history.remove(0);
                                    }
                                    history.push(scan_result);
                                },
                                Err(e) => eprintln!("Failed to scan file {}: {}", file_path, e),
                            }
                        }).await;
                    }
                }

                // Sleep before next scan cycle
                sleep(Duration::from_secs(3600)).await;
            }
        });
    }
}
