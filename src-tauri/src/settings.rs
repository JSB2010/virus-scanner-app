use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub api_key: String,
    pub welcome_completed: bool,
    pub auto_scan_downloads: bool,
    pub notify_on_scan_completion: bool,
    pub auto_quarantine_malicious: bool,
    pub auto_rescan_interval: Option<u64>, // Hours between automatic rescans
    pub custom_scan_locations: Vec<String>,
    pub quarantine_settings: QuarantineSettings,
    pub scan_history_limit: u32,
    pub file_type_filters: Vec<String>,
    pub theme: String,
    pub startup_with_system: bool,
    pub minimize_to_tray: bool,
    pub export_path: Option<String>,
    pub background_scan_threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineSettings {
    pub auto_delete_after_days: Option<u32>,
    pub backup_before_quarantine: bool,
    pub quarantine_location: Option<String>,
    pub notify_on_quarantine: bool,
}

impl Default for QuarantineSettings {
    fn default() -> Self {
        Self {
            auto_delete_after_days: Some(30),
            backup_before_quarantine: true,
            quarantine_location: None,
            notify_on_quarantine: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            welcome_completed: false,
            auto_scan_downloads: true,
            notify_on_scan_completion: true,
            auto_quarantine_malicious: false,
            auto_rescan_interval: Some(24), // Default to daily rescans
            custom_scan_locations: Vec::new(),
            quarantine_settings: QuarantineSettings::default(),
            scan_history_limit: 1000,
            file_type_filters: vec![
                "exe".to_string(), "dll".to_string(), "bat".to_string(),
                "cmd".to_string(), "ps1".to_string(), "vbs".to_string(),
                "js".to_string(), "jar".to_string(), "msi".to_string(),
                "zip".to_string(), "rar".to_string(), "scr".to_string(),
                "pdf".to_string(), "doc".to_string(), "docx".to_string()
            ],
            theme: "system".to_string(),
            startup_with_system: false,
            minimize_to_tray: true,
            export_path: None,
            background_scan_threads: 2,
        }
    }
}

impl Settings {
    #[allow(dead_code)]
    pub fn unwrap_or_default(self) -> Self {
        self
    }

    pub fn load() -> Result<Self, String> {
        println!("Loading settings");

        // Use a proper config directory
        let config_dir = if let Some(app_dir) = dirs::config_dir() {
            app_dir.join("virus-scanner-app")
        } else {
            PathBuf::from("./config")
        };

        let settings_path = config_dir.join("settings.json");
        println!("Settings path: {}", settings_path.display());

        if settings_path.exists() {
            match fs::read_to_string(&settings_path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(settings) => {
                            println!("Settings loaded successfully");
                            return Ok(settings);
                        },
                        Err(e) => {
                            println!("Failed to parse settings: {}, using defaults", e);
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to read settings file: {}, using defaults", e);
                }
            }
        } else {
            println!("Settings file does not exist, using defaults");
        }

        // Return default settings if loading fails
        let default_settings = Self::default();
        println!("Using default settings");
        Ok(default_settings)
    }

    pub fn save(&self) -> Result<(), String> {
        println!("Saving settings");

        // Use a proper config directory
        let config_dir = if let Some(app_dir) = dirs::config_dir() {
            app_dir.join("virus-scanner-app")
        } else {
            PathBuf::from("./config")
        };

        println!("Config directory: {}", config_dir.display());

        // Create the config directory if it doesn't exist
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let settings_path = config_dir.join("settings.json");

        // Serialize the settings to JSON
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        // Write the settings to the file
        fs::write(&settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        println!("Settings saved successfully to: {}", settings_path.display());
        Ok(())
    }
}

// Tauri commands for settings
#[tauri::command]
pub async fn get_settings(_state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    Ok(Settings::load()?)
}

#[tauri::command]
pub async fn update_settings(settings: Settings, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    settings.save()
}

#[tauri::command]
pub async fn reset_settings() -> Result<Settings, String> {
    let default_settings = Settings::default();
    default_settings.save()?;
    Ok(default_settings)
}

#[tauri::command]
pub async fn get_quarantine_settings() -> Result<QuarantineSettings, String> {
    let settings = Settings::load()?;
    Ok(settings.quarantine_settings)
}

#[tauri::command]
pub async fn update_quarantine_settings(quarantine: QuarantineSettings, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut settings = Settings::load()?;
    settings.quarantine_settings = quarantine;
    settings.save()
}

// Export settings and history
#[tauri::command]
#[allow(dead_code)]
pub async fn export_data(app_state: tauri::State<'_, AppState>) -> Result<String, String> {
    #[derive(Serialize)]
    #[allow(dead_code)]
    struct ExportData {
        #[allow(dead_code)]
        settings: Settings,
        #[allow(dead_code)]
        scan_history: Vec<crate::virus_total::ScanResult>,
        #[allow(dead_code)]
        export_date: String,
    }

    let settings = Settings::load()?;
    let scan_history = app_state.scan_history.lock().await.clone();

    let export_data = ExportData {
        settings,
        scan_history,
        export_date: chrono::Local::now().to_rfc3339(),
    };

    serde_json::to_string_pretty(&export_data)
        .map_err(|e| format!("Failed to serialize export data: {}", e))
}

// Import settings and history
#[tauri::command]
#[allow(dead_code)]
pub async fn import_data(
    data: String,
    app_state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct ExportData {
        #[allow(dead_code)]
        settings: Settings,
        #[allow(dead_code)]
        scan_history: Vec<crate::virus_total::ScanResult>,
        #[allow(dead_code)]
        export_date: String,
    }

    // Parse the import data
    let import_data: ExportData = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse import data: {}", e))?;

    // Update settings
    import_data.settings.save()?;

    // Update scan history
    let mut history = app_state.scan_history.lock().await;
    *history = import_data.scan_history;

    Ok(())
}
