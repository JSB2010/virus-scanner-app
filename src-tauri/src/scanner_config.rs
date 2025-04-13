use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub monitored_extensions: Vec<String>,
    pub scan_batch_size: usize,
    pub max_concurrent_scans: usize,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            monitored_extensions: vec![
                "exe".to_string(), "dll".to_string(), "sys".to_string(),
                "msi".to_string(), "bat".to_string(), "cmd".to_string(),
                "ps1".to_string(), "vbs".to_string(), "js".to_string(),
                "jar".to_string(), "zip".to_string(), "rar".to_string(),
            ],
            scan_batch_size: 5,
            max_concurrent_scans: 3,
            retry_attempts: 3,
            retry_delay_seconds: 60,
        }
    }
}
