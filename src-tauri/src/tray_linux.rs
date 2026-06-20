//! Native StatusNotifierItem tray for Linux, via `ksni`.
//!
//! Tauri's built-in tray (the `tray-icon` crate) uses libayatana-appindicator
//! on Linux, which never forwards a left-click / SNI `Activate` event to the
//! app — only the context menu works. By implementing the StatusNotifierItem
//! ourselves (the same approach Qt apps such as Nextcloud use) we get proper
//! left-click-to-toggle behaviour. The tray runs in its own background thread.

use ksni::{
    menu::{MenuItem, StandardItem},
    Tray, TrayService,
};
use tauri::{AppHandle, Manager};

/// Toggle the main window between shown/focused and hidden.
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

struct CinnyTray {
    app: AppHandle,
}

impl Tray for CinnyTray {
    fn id(&self) -> String {
        "in.cinny.app".into()
    }

    fn title(&self) -> String {
        "Cinny".into()
    }

    // Resolved from the installed hicolor theme (/usr/share/icons/.../cinny.png).
    fn icon_name(&self) -> String {
        "cinny".into()
    }

    // Left click.
    fn activate(&mut self, _x: i32, _y: i32) {
        toggle_window(&self.app);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Show/Hide Cinny".into(),
                activate: Box::new(|this: &mut Self| toggle_window(&this.app)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| this.app.exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Spawn the native SNI tray on its own thread.
pub fn build(app: AppHandle) {
    let service = TrayService::new(CinnyTray { app });
    service.spawn();
}
