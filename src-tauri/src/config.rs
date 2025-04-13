use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub enabled: bool,
    pub scan_interval: Duration,
    pub batch_size: usize,
    pub max_concurrent_scans: usize,
    pub retry_attempts: usize,
    pub retry_delay: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub enabled: bool,
    pub monitored_paths: Vec<PathBuf>,
    pub excluded_paths: Vec<PathBuf>,
    pub file_extensions: Vec<String>,
    pub min_file_size: u64,
    pub max_file_size: u64,
    pub debounce_duration: Duration,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_interval: Duration::from_secs(3600), // 1 hour
            batch_size: 10,
            max_concurrent_scans: 2,
            retry_attempts: 3,
            retry_delay: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            monitored_paths: vec![],
            excluded_paths: vec![],
            file_extensions: vec![
                "exe".to_string(), "dll".to_string(), "sys".to_string(),
                "msi".to_string(), "zip".to_string(), "rar".to_string(),
            ],
            min_file_size: 1024, // 1KB
            max_file_size: 100 * 1024 * 1024, // 100MB
            debounce_duration: Duration::from_secs(1),
        }
    }
}

impl MonitorConfig {
    pub fn should_monitor_file(&self, path: &PathBuf) -> bool {
        // Check if monitoring is enabled
        if !self.enabled {
            return false;
        }

        // Check if path is in excluded paths
        if self.excluded_paths.iter().any(|excluded| path.starts_with(excluded)) {
            return false;
        }

        // Check if path is in monitored paths
        if !self.monitored_paths.iter().any(|monitored| path.starts_with(monitored)) {
            return false;
        }

        // Check file extension
        if let Some(ext) = path.extension() {
            if !self.file_extensions.iter().any(|allowed_ext| 
                allowed_ext.eq_ignore_ascii_case(&ext.to_string_lossy())
            ) {
                return false;
            }
        } else {
            return false;
        }

        // Check file size if file exists
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();
            if size < self.min_file_size || size > self.max_file_size {
                return false;
            }
        }

        true
    }
}
