use std::path::{Path, PathBuf};
use std::collections::HashMap;
use notify::{Watcher, RecursiveMode, EventKind};
use tauri::{WebviewWindow, Emitter};
use crate::settings::Settings;

pub struct FileMonitor {
    is_monitoring: bool,
    download_path: PathBuf,
    settings: Settings,
}

impl Default for FileMonitor {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl FileMonitor {
    pub fn new(settings: Settings) -> Self {
        let download_path = dirs::download_dir().unwrap_or_else(|| PathBuf::from("."));
        
        FileMonitor {
            is_monitoring: false,
            download_path,
            settings,
        }
    }
    
    fn should_monitor_file(path: &Path, settings: &Settings) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return settings.file_type_filters.contains(&ext_str.to_lowercase());
            }
        }
        // If no extension or not in filters, use default behavior
        path.is_file()
    }
    
    pub async fn start_monitoring(
        &mut self,
        window: WebviewWindow,
    ) -> Result<(), String> {
        println!("Starting file monitoring");
        
        if self.is_monitoring {
            return Ok(());
        }
        
        self.is_monitoring = true;
        
        // Create a channel for the watcher to send events
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Create a watcher
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| format!("Failed to create watcher: {}", e))?;
        
        // Watch the downloads directory
        watcher.watch(&self.download_path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch downloads directory: {}", e))?;
        
        println!("Monitoring directory: {}", self.download_path.display());
        
        // Clone window and settings for the async task
        let window_clone = window.clone();
        let settings = self.settings.clone();
        
        // Spawn a task to handle file events
        tokio::spawn(async move {
            let mut watcher_map = HashMap::new();
            watcher_map.insert("downloads".to_string(), watcher);
            
            // Process events from the watcher
            while let Ok(event_result) = rx.recv() {
                match event_result {
                    Ok(event) => {
                        if let EventKind::Create(_) = event.kind {
                            for path in event.paths {
                                if path.is_file() && Self::should_monitor_file(&path, &settings) {
                                    let _file_name = path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown file")
                                        .to_string();
                                    
                                    println!("New file detected: {}", path.display());
                                    
                                    // Emit an event to the frontend
                                    if let Err(e) = window_clone.emit("file-detected", path.to_string_lossy().to_string()) {
                                        eprintln!("Failed to emit file-detected event: {}", e);
                                    }
                                    
                                    // Show a notification
                                    if let Err(e) = window_clone.emit("new-file-detected", path.to_string_lossy().to_string()) {
                                        eprintln!("Failed to emit new-file-detected event: {}", e);
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Watch error: {:?}", e),
                }
            }
            
            println!("File monitoring stopped");
        });
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn stop_monitoring(&mut self) -> Result<(), String> {
        println!("Stopping file monitoring");
        self.is_monitoring = false;
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn is_monitoring(&self) -> bool {
        self.is_monitoring
    }
    
    pub fn get_download_path(&self) -> &Path {
        &self.download_path
    }
    
    pub fn set_download_path(&mut self, path: PathBuf) {
        self.download_path = path;
    }
    
    #[allow(dead_code)]
    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }
}

// Tauri commands for file monitoring
#[tauri::command]
pub async fn get_download_path(
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let file_monitor = state.file_monitor.lock().await;
    Ok(file_monitor.get_download_path().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn set_download_path(
    path: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let mut file_monitor = state.file_monitor.lock().await;
    let path_buf = PathBuf::from(path);
    
    if !path_buf.exists() || !path_buf.is_dir() {
        return Err("Invalid directory path".to_string());
    }
    
    file_monitor.set_download_path(path_buf);
    Ok(())
}

#[tauri::command]
pub async fn scan_downloads_folder(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let file_monitor = state.file_monitor.lock().await;
    let download_path = file_monitor.get_download_path().to_string_lossy().to_string();
    
    // Scan all files in the downloads folder
    let entries = std::fs::read_dir(download_path)
        .map_err(|e| format!("Failed to read downloads directory: {}", e))?;
    
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                // Emit an event to scan this file
                println!("Found file to scan: {}", path.display());
                // We'll let the frontend handle the actual scanning
            }
        }
    }
    
    Ok(())
}
