use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub enabled_paths: Vec<String>,
    pub watch_timeout: Duration,
    pub debounce_duration: Duration,
    pub max_watch_recursion_depth: u32,
    pub ignored_patterns: Vec<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled_paths: Vec::new(),
            watch_timeout: Duration::from_secs(30),
            debounce_duration: Duration::from_millis(500),
            max_watch_recursion_depth: 5,
            ignored_patterns: vec![
                String::from("**/node_modules/**"),
                String::from("**/.git/**"),
                String::from("**/target/**"),
                String::from("**/*.tmp")
            ],
        }
    }
}

impl MonitorConfig {
    pub fn should_monitor_path(&self, path: &str) -> bool {
        // Skip paths matching ignore patterns
        for pattern in &self.ignored_patterns {
            if glob::Pattern::new(pattern).map(|p| p.matches(path)).unwrap_or(false) {
                return false;
            }
        }
        true
    }
}
