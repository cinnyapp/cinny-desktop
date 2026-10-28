//! Native StatusNotifierItem tray for Linux, via `ksni`.
//!
//! Tauri's built-in tray (the `tray-icon` crate) uses libayatana-appindicator
//! on Linux, which never forwards a left-click / SNI `Activate` event to the
//! app — only the context menu works. By implementing the StatusNotifierItem
//! ourselves (the same approach Qt apps such as Nextcloud use) we get proper
//! left-click-to-toggle behaviour. The tray runs in its own background thread.

use ksni::{
    menu::{MenuItem, StandardItem},
    Handle, Icon, Tray, TrayService,
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
    /// Icon with the unread dot (ARGB32), shown instead of the themed icon
    /// while there are unread messages.
    unread_icon: Option<Icon>,
    unread: bool,
}

impl Tray for CinnyTray {
    fn id(&self) -> String {
        "in.cinny.app".into()
    }

    fn title(&self) -> String {
        "Cinny".into()
    }

    // Resolved from the installed hicolor theme (/usr/share/icons/.../cinny.png).
    // Hosts prefer icon_name over icon_pixmap, so it is cleared while unread.
    fn icon_name(&self) -> String {
        if self.unread && self.unread_icon.is_some() {
            String::new()
        } else {
            "cinny".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        match (&self.unread_icon, self.unread) {
            (Some(icon), true) => vec![icon.clone()],
            _ => vec![],
        }
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
    let unread_icon = app.default_window_icon().map(|icon| {
        let icon = crate::badge::with_dot(icon);
        // RGBA -> ARGB
        let mut data = icon.rgba().to_vec();
        for px in data.chunks_exact_mut(4) {
            px.rotate_right(1);
        }
        Icon {
            width: icon.width() as i32,
            height: icon.height() as i32,
            data,
        }
    });
    let service = TrayService::new(CinnyTray {
        app: app.clone(),
        unread_icon,
        unread: false,
    });
    app.manage(service.handle());
    service.spawn();
}

/// Show or hide the unread dot on the tray icon.
pub fn set_unread(app: &AppHandle, unread: bool) {
    if let Some(handle) = app.try_state::<Handle<CinnyTray>>() {
        handle.update(|tray| tray.unread = unread);
    }
}
