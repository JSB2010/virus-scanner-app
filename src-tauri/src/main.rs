#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod settings;
mod virus_total;
mod file_monitor;

use std::sync::Arc;
use tauri::{State, Manager};
use tokio::sync::Mutex;
use settings::Settings;
use file_monitor::FileMonitor;
use virus_total::ScanResult;

#[derive(Default)]
pub struct AppState {
    api_key: Arc<Mutex<Option<String>>>,
    file_monitor: Arc<Mutex<FileMonitor>>,
    scan_history: Arc<Mutex<Vec<ScanResult>>>,
    is_setup_complete: Arc<Mutex<bool>>,
}

#[tauri::command]
async fn initialize_api(
    api_key: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // Test the API key first
    let is_valid = virus_total::test_api_key(api_key.clone()).await?;

    if !is_valid {
        return Err("Invalid API key".to_string());
    }

    // Store the API key
    *state.api_key.lock().await = Some(api_key.clone());

    // Initialize file monitor with default settings
    let settings = Settings::default();
    let mut monitor = state.file_monitor.lock().await;
    *monitor = FileMonitor::new(settings);

    // Mark setup as complete
    *state.is_setup_complete.lock().await = true;

    Ok(true)
}

#[tauri::command]
async fn start_monitoring(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Check if setup is complete
    let is_setup_complete = *state.is_setup_complete.lock().await;
    if !is_setup_complete {
        return Err("Setup not complete".to_string());
    }

    // Start monitoring
    let mut monitor = state.file_monitor.lock().await;
    monitor.start_monitoring(window).await
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Ensure the main window is created and visible
            let main_window = app.get_webview_window("main").unwrap();
            main_window.show().unwrap();
            main_window.set_focus().unwrap();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide the window instead of closing it
                window.hide().unwrap_or_default();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            initialize_api,
            start_monitoring,
            virus_total::scan_file,
            virus_total::delete_file,
            virus_total::test_api_key,
            virus_total::is_setup_complete,
            virus_total::get_scan_history,
            virus_total::clear_scan_history,
            file_monitor::get_download_path,
            file_monitor::set_download_path,
            file_monitor::scan_downloads_folder,
            settings::get_settings,
            settings::update_settings,
            settings::reset_settings,
            settings::get_quarantine_settings,
            settings::update_quarantine_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
