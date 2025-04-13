use tauri::{AppHandle, Manager, Runtime, Emitter};
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

pub fn create_tray<R: Runtime>() -> TrayIconBuilder<R> {
    let menu_builder = MenuBuilder::new();
    let menu = menu_builder
        .item(&MenuItemBuilder::new("Show").build().unwrap())
        .item(&MenuItemBuilder::new("Scan File").build().unwrap())
        .item(&MenuItemBuilder::new("Settings").build().unwrap())
        .item(&MenuItemBuilder::new("History").build().unwrap())
        .separator()
        .item(&MenuItemBuilder::new("Quit").build().unwrap())
        .build()
        .unwrap();

    TrayIconBuilder::new().menu(&menu)
}

pub fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { 
            button: MouseButton::Left, 
            button_state: MouseButtonState::Up, 
            .. 
        } => {
            show_main_window(app);
        },
        TrayIconEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                "Quit" => {
                    app.exit(0);
                }
                "Show" => {
                    show_main_window(app);
                }
                "Scan File" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.emit("show-file-dialog", ()).unwrap();
                    }
                }
                "Settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.emit("show-settings", ()).unwrap();
                    }
                    show_main_window(app);
                }
                "History" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.emit("show-history", ()).unwrap();
                    }
                    show_main_window(app);
                }
                _ => {}
            }
        },
        _ => {}
    }
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
        
        #[cfg(target_os = "macos")]
        window.eval("if (window.__TAURI_METADATA__) { window.__TAURI_METADATA__.__tauriUseMacOS = true; }").unwrap();
    }
}
