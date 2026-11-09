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
    menu::{CheckmarkItem, MenuItem, StandardItem},
    Handle, Icon, Tray, TrayService,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use dbus::{
    blocking::{stdintf::org_freedesktop_dbus::Properties, Connection},
    message::MatchRule,
    Message,
};
use tauri::{AppHandle, Manager};

use crate::{
    settings,
    window::{self, toggle_main},
};

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

    // Keep the service running while there is no watcher, in case one
    // appears later. Availability is tracked by `watch_host`.
    fn watcher_offine(&self) -> bool {
        true
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let current = settings::get(&self.app);
        let autostart_on = settings::autostart_enabled(&self.app);

        let mut items = vec![
            StandardItem {
                label: "Show/Hide Cinny".into(),
                activate: Box::new(|this: &mut Self| on_main_thread(&this.app, toggle_main)),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            CheckmarkItem {
                label: "Close to tray".into(),
                checked: current.close_to_tray,
                activate: Box::new(|this: &mut Self| {
                    settings::update(&this.app, |s| s.close_to_tray = !s.close_to_tray)
                }),
                ..Default::default()
            }
            .into(),
        ];
        if settings::autostart_supported() {
            items.push(
                CheckmarkItem {
                    label: "Launch at login".into(),
                    checked: autostart_on,
                    activate: Box::new(|this: &mut Self| {
                        settings::set_autostart(&this.app, !settings::autostart_enabled(&this.app));
                    }),
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                CheckmarkItem {
                    label: "Start hidden at login".into(),
                    checked: current.start_hidden,
                    enabled: autostart_on,
                    activate: Box::new(|this: &mut Self| {
                        settings::update(&this.app, |s| s.start_hidden = !s.start_hidden)
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }
        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|this: &mut Self| this.app.exit(0)),
                ..Default::default()
            }
            .into(),
        );
        items
    }
}

/// Run `f` on the main thread without waiting for it.
fn on_main_thread(app: &AppHandle, f: fn(&AppHandle)) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || f(&handle));
}

/// Tray state on Linux. Kept under one lock so a tray failure cannot be
/// overwritten by a host change that was read just before it.
struct TrayState {
    host_registered: bool,
    failed: bool,
    /// Bumped on every change, so a pending grace check knows it is stale.
    generation: u64,
}

static STATE: Mutex<TrayState> = Mutex::new(TrayState {
    host_registered: false,
    failed: false,
    generation: 0,
});

/// Time a tray host gets to appear, e.g. when Cinny starts at login before
/// the panel, before a hidden window is shown.
const HOST_GRACE: Duration = Duration::from_secs(10);

const WATCHER: &str = "org.kde.StatusNotifierWatcher";

/// Track whether a tray host shows our icon, and bring back a hidden window
/// if there is none. ksni only reports the watcher, which can run without a
/// host (e.g. Plasma with the system tray widget removed).
fn watch_host(app: AppHandle) {
    std::thread::spawn(move || {
        let conn = match Connection::new_session() {
            Ok(conn) => conn,
            Err(err) => {
                eprintln!("Cannot watch the tray host: {err}");
                update(&app, |state| state.host_registered = false);
                return;
            }
        };
        let changed = Arc::new(AtomicBool::new(true));
        let mut subscribed = true;
        for rule in [
            MatchRule::new_signal(WATCHER, "StatusNotifierHostRegistered"),
            MatchRule::new_signal(WATCHER, "StatusNotifierHostUnregistered"),
            MatchRule::new_signal("org.freedesktop.DBus.Properties", "PropertiesChanged")
                .with_sender(WATCHER),
        ] {
            let changed = changed.clone();
            subscribed &= conn
                .add_match(rule, move |_: (), _: &Connection, _: &Message| {
                    changed.store(true, Ordering::Relaxed);
                    true
                })
                .is_ok();
        }
        // The watcher itself appearing or going away.
        let owner_changed = changed.clone();
        subscribed &= conn
            .add_match(
                MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged"),
                move |(name, _, _): (String, String, String), _: &Connection, _: &Message| {
                    if name == WATCHER {
                        owner_changed.store(true, Ordering::Relaxed);
                    }
                    true
                },
            )
            .is_ok();
        if !subscribed {
            // Without change signals the state could go stale; treat the tray
            // as unavailable rather than risk hiding the window for good.
            eprintln!("Cannot watch the tray host: subscribing to D-Bus signals failed");
            update(&app, |state| state.host_registered = false);
            return;
        }

        let mut last = None;
        loop {
            if changed.swap(false, Ordering::Relaxed) {
                let registered = conn
                    .with_proxy(WATCHER, "/StatusNotifierWatcher", Duration::from_secs(2))
                    .get::<bool>(WATCHER, "IsStatusNotifierHostRegistered")
                    .unwrap_or(false);
                if last != Some(registered) {
                    last = Some(registered);
                    update(&app, |state| state.host_registered = registered);
                }
            }
            if let Err(err) = conn.process(Duration::from_secs(60)) {
                eprintln!("Stopped watching the tray host: {err}");
                update(&app, |state| state.host_registered = false);
                return;
            }
        }
    });
}

/// Change the tray state and publish whether the tray is usable. If it is
/// not, a hidden window is shown unless a host appears within `HOST_GRACE`.
fn update(app: &AppHandle, change: impl FnOnce(&mut TrayState)) {
    let mut state = STATE.lock().unwrap();
    change(&mut state);
    state.generation += 1;
    let available = state.host_registered && !state.failed;
    window::set_tray_available(available);
    if available {
        return;
    }
    let generation = state.generation;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(HOST_GRACE);
        if STATE.lock().unwrap().generation == generation {
            on_main_thread(&app, window::show_if_hidden);
        }
    });
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
    watch_host(app.clone());
    // Flatpak apps may not own the per-process D-Bus name ksni requests.
    let in_flatpak = std::path::Path::new("/.flatpak-info").exists();
    std::thread::spawn(move || {
        let result = if in_flatpak {
            service.run_without_dbus_name()
        } else {
            service.run()
        };
        if let Err(err) = result {
            eprintln!("System tray unavailable: {err}");
            window::set_tray_available(false);
            on_main_thread(&app, window::show_if_hidden);
            update(&app, |state| state.failed = true);
        }
    });
}

/// Show or hide the unread dot on the tray icon.
pub fn set_unread(app: &AppHandle, unread: bool) {
    if let Some(handle) = app.try_state::<Handle<CinnyTray>>() {
        handle.update(|tray| tray.unread = unread);
    }
}
