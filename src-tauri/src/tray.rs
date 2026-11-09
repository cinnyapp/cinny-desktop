use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

use crate::{
    settings,
    window::{show_main, toggle_main},
};

/// Menu entries whose state changes at runtime.
struct TrayMenu<R: Runtime> {
    close_to_tray: CheckMenuItem<R>,
    autostart: Option<CheckMenuItem<R>>,
    start_hidden: Option<CheckMenuItem<R>>,
}

/// Build the system tray.
///
/// On Windows, left-clicking the icon toggles the main window and the menu
/// opens on right click. On macOS any click opens the menu, as is usual for
/// menu bar items. The menu offers a show/hide toggle, the desktop settings
/// and a quit entry.
///
/// This restores the tray that was added in #166 and temporarily reverted in
/// #312 (a Flatpak/single-instance packaging issue, unrelated to the tray
/// itself), ported to Tauri v2.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let current = settings::get(app);
    let autostart_on = settings::autostart_enabled(app);

    let toggle = MenuItem::with_id(app, "toggle", "Show/Hide Cinny", true, None::<&str>)?;
    let close_to_tray = CheckMenuItem::with_id(
        app,
        "close_to_tray",
        "Close to tray",
        true,
        current.close_to_tray,
        None::<&str>,
    )?;
    let (autostart, start_hidden) = if settings::autostart_supported() {
        (
            Some(CheckMenuItem::with_id(
                app,
                "autostart",
                "Launch at login",
                true,
                autostart_on,
                None::<&str>,
            )?),
            Some(CheckMenuItem::with_id(
                app,
                "start_hidden",
                "Start hidden at login",
                autostart_on,
                current.start_hidden,
                None::<&str>,
            )?),
        )
    } else {
        (None, None)
    };
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let separator = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&toggle, &separator, &close_to_tray])?;
    if let (Some(autostart), Some(start_hidden)) = (&autostart, &start_hidden) {
        menu.append_items(&[autostart, start_hidden])?;
    }
    menu.append_items(&[&separator2, &quit])?;

    app.manage(TrayMenu {
        close_to_tray,
        autostart,
        start_hidden,
    });

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Cinny")
        .menu(&menu)
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "toggle" => toggle_main(app),
            "close_to_tray" => {
                settings::update(app, |s| s.close_to_tray = !s.close_to_tray);
                sync_menu(app);
            }
            "autostart" => {
                settings::set_autostart(app, !settings::autostart_enabled(app));
                sync_menu(app);
            }
            "start_hidden" => {
                settings::update(app, |s| s.start_hidden = !s.start_hidden);
                sync_menu(app);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // macOS opens the menu on click instead.
            if cfg!(target_os = "macos") {
                return;
            }
            match event {
                // Toggle on press: a double click sends press, release,
                // double click, release, so it ends with the window shown.
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Down,
                    ..
                } => toggle_main(tray.app_handle()),
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => show_main(tray.app_handle()),
                _ => {}
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

/// Set the check marks from the stored settings. Check items toggle
/// themselves on click, so this also corrects them if a change failed.
fn sync_menu<R: Runtime>(app: &AppHandle<R>) {
    let Some(menu) = app.try_state::<TrayMenu<R>>() else {
        return;
    };
    let current = settings::get(app);
    let autostart_on = settings::autostart_enabled(app);
    let _ = menu.close_to_tray.set_checked(current.close_to_tray);
    if let Some(item) = &menu.autostart {
        let _ = item.set_checked(autostart_on);
    }
    if let Some(item) = &menu.start_hidden {
        let _ = item.set_checked(current.start_hidden);
        let _ = item.set_enabled(autostart_on);
    }
}

/// Swap the tray icon for a variant with an unread dot, or back.
pub fn set_unread<R: Runtime>(app: &AppHandle<R>, unread: bool) {
    let (Some(tray), Some(icon)) = (app.tray_by_id("main-tray"), app.default_window_icon()) else {
        return;
    };
    let icon = if unread {
        crate::badge::with_dot(icon)
    } else {
        icon.clone().to_owned()
    };
    let _ = tray.set_icon(Some(icon));
}
