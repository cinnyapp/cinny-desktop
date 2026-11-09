//! Helpers for showing and hiding the main window, shared by the tray,
//! single instance and the macOS Dock.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, Runtime};

/// Whether a tray icon is shown. On Linux this needs a StatusNotifierItem
/// host (e.g. GNOME without the AppIndicator extension has none), so it
/// starts false there and is set once a tray host is registered.
static TRAY_AVAILABLE: AtomicBool = AtomicBool::new(cfg!(not(target_os = "linux")));

pub fn set_tray_available(available: bool) {
    TRAY_AVAILABLE.store(available, Ordering::Relaxed);
}

/// Closing the window only hides it if there is a tray icon to bring it
/// back; otherwise the window could not be reopened.
pub fn tray_available() -> bool {
    TRAY_AVAILABLE.load(Ordering::Relaxed)
}

/// Show, unminimize and focus the main window.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    // The taskbar overlay is lost while the window is hidden.
    crate::badge::reapply(app);
}

/// Bring back a hidden window when there is no tray icon to reopen it from.
pub fn show_if_hidden<R: Runtime>(app: &AppHandle<R>) {
    let hidden = app
        .get_webview_window("main")
        .is_some_and(|window| !window.is_visible().unwrap_or(true));
    if hidden {
        show_main(app);
    }
}

/// Hide the main window if it is on screen, otherwise bring it back.
///
/// `is_visible` is also true for a minimized window, so a minimized window
/// counts as hidden here and gets restored instead of hidden. Wayland does
/// not report minimized windows, so there a minimized window is hidden on
/// the first click and restored on the second.
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
