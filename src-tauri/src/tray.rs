use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Runtime,
};

use crate::window::{show_main, toggle_main};

/// Build the system tray.
///
/// On Windows, left-clicking the icon toggles the main window and the menu
/// opens on right click. On macOS any click opens the menu, as is usual for
/// menu bar items. The menu offers a show/hide toggle and a quit entry.
///
/// This restores the tray that was added in #166 and temporarily reverted in
/// #312 (a Flatpak/single-instance packaging issue, unrelated to the tray
/// itself), ported to Tauri v2.
pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Show/Hide Cinny", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Cinny")
        .menu(&menu)
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "toggle" => toggle_main(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // macOS opens the menu on click instead.
            if cfg!(target_os = "macos") {
                return;
            }
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => toggle_main(tray.app_handle()),
                // A double click also sends two clicks; make sure it ends shown.
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
