use tauri::{AppHandle, Manager};
use serde_json::json;
use crate::settings::Settings;
use crate::AppState;

pub struct SystemIntegration {
    app_handle: AppHandle,
}

impl SystemIntegration {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn setup_tray_menu(&self) {
        if let Some(window) = self.app_handle.get_window("main") {
            window.on_window_event(|event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                    #[cfg(not(target_os = "macos"))]
                    {
                        window.hide().unwrap();
                        api.prevent_close();
                    }
                }
            });
        }
    }

    pub async fn show_notification(&self, title: &str, body: &str) -> Result<(), String> {
        // First try to use system notifications
        #[cfg(not(target_os = "linux"))]
        {
            use tauri::notification::Notification;
            
            if let Err(e) = Notification::new(&self.app_handle.config().tauri.bundle.identifier)
                .title(title)
                .body(body)
                .icon("icon.png")
                .show() 
            {
                eprintln!("Failed to show system notification: {}", e);
                // Fall back to in-app notification
                self.show_in_app_notification(title, body).await?;
            }
        }

        // On Linux or if system notification failed, show in-app notification
        #[cfg(target_os = "linux")]
        {
            self.show_in_app_notification(title, body).await?;
        }

        Ok(())
    }

    async fn show_in_app_notification(&self, title: &str, body: &str) -> Result<(), String> {
        self.app_handle.emit_all("notification", json!({
            "title": title,
            "body": body,
        })).map_err(|e| format!("Failed to emit notification event: {}", e))?;

        Ok(())
    }

    pub async fn handle_startup(&self, state: &AppState) -> Result<(), String> {
        // Load settings
        let settings = Settings::load().unwrap_or_default();
        
        // Start file monitoring if enabled
        if settings.auto_scan_downloads {
            self.app_handle.emit_all("start-monitoring", ())
                .map_err(|e| format!("Failed to emit start-monitoring event: {}", e))?;
        }
        
        // Show the main window based on settings
        if let Some(window) = self.app_handle.get_window("main") {
            if !settings.minimize_to_tray {
                window.show()
                    .map_err(|e| format!("Failed to show window: {}", e))?;
            }
        }

        Ok(())
    }
}
