//! Helpers for showing and hiding the main window, shared by the tray,
//! single instance and the macOS Dock.

use tauri::{AppHandle, Manager, Runtime};

/// Show, unminimize and focus the main window.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hide the main window if it is on screen, otherwise bring it back.
///
/// `is_visible` is also true for a minimized window, so a minimized window
/// counts as hidden here and gets restored instead of hidden.
pub fn toggle_main<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let on_screen =
        window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false);
    if on_screen {
        let _ = window.hide();
    } else {
        show_main(app);
    }
}
