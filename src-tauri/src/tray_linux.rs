//! Native StatusNotifierItem tray for Linux, via `ksni`.
//!
//! Tauri's built-in tray (the `tray-icon` crate) uses libayatana-appindicator
//! on Linux, which never forwards a left-click / SNI `Activate` event to the
//! app — only the context menu works. By implementing the StatusNotifierItem
//! ourselves (the same approach Qt apps such as Nextcloud use) we get proper
//! left-click-to-toggle behaviour. The tray runs in its own background thread.
//!
//! ksni holds a lock on the tray while a callback runs, and `set_unread` takes
//! that lock from the main thread. Callbacks therefore must not block on the
//! main thread: window calls are dispatched with `run_on_main_thread`.

use ksni::{
    menu::{MenuItem, StandardItem},
    Handle, Icon, Tray, TrayService,
};
use tauri::{AppHandle, Manager};

use crate::window::{self, toggle_main};

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
        on_main_thread(&self.app, toggle_main);
    }

    fn watcher_online(&self) {
        window::set_tray_available(true);
    }

    // No tray host: keep the service running in case one appears later, but
    // let closing the window quit instead of hiding it.
    fn watcher_offine(&self) -> bool {
        window::set_tray_available(false);
        true
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: "Show/Hide Cinny".into(),
                activate: Box::new(|this: &mut Self| on_main_thread(&this.app, toggle_main)),
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

/// Run `f` on the main thread without waiting for it.
fn on_main_thread(app: &AppHandle, f: fn(&AppHandle)) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || f(&handle));
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
    // Flatpak apps may not own the per-process D-Bus name ksni requests.
    let in_flatpak = std::path::Path::new("/.flatpak-info").exists();
    std::thread::spawn(move || {
        let result = if in_flatpak {
            service.run_without_dbus_name()
        } else {
            service.run()
        };
        if let Err(err) = result {
            window::set_tray_available(false);
            eprintln!("System tray unavailable: {err}");
        }
    });
}

/// Show or hide the unread dot on the tray icon.
pub fn set_unread(app: &AppHandle, unread: bool) {
    if let Some(handle) = app.try_state::<Handle<CinnyTray>>() {
        handle.update(|tray| tray.unread = unread);
    }
}
