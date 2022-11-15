use tauri::{
    CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowEvent,
};

#[cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#[cfg(target_os = "macos")]
mod menu;

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    builder
        .system_tray(system_tray())
        .on_system_tray_event(system_tray_handler)
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(run_event_handler)
}

fn run_event_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            // Prevent Cinny from closing, instead hide it and let it be
            // reopened through the tray.
            api.prevent_close();
            app.get_window(&label).unwrap().hide().unwrap();
            app.tray_handle_by_id("main-tray")
                .unwrap()
                .get_item("toggle")
                .set_title("Show Cinny")
                .unwrap();
        }
        _ => {}
    }
}

fn system_tray() -> SystemTray {
    let toggle = CustomMenuItem::new("toggle".to_owned(), "Hide Cinny");
    let quit = CustomMenuItem::new("quit".to_owned(), "Quit");
    let menu = SystemTrayMenu::new()
        .add_item(toggle)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    tauri::SystemTray::new()
        .with_menu(menu)
        .with_id("main-tray".to_owned())
}

fn system_tray_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: SystemTrayEvent) {
    let tray_handle = match app.tray_handle_by_id("main-tray") {
        Some(h) => h,
        None => return,
    };

    match event {
        SystemTrayEvent::LeftClick { .. } => {
            // Show the window if it's hidden or whatever
            app.get_window("main").unwrap().show().ok();
            tray_handle
                .get_item("toggle")
                .set_title("Hide Cinny")
                .unwrap();
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            let item_handle = tray_handle.get_item(&id);
            match id.as_str() {
                "quit" => {
                    app.exit(0);
                }
                "toggle" => {
                    let window = app.get_window("main").unwrap();
                    // Hide the window if it's visible, show it if not
                    let new_title = if window.is_visible().unwrap() {
                        window.hide().unwrap();
                        "Show Cinny"
                    } else {
                        window.show().unwrap();
                        "Hide Cinny"
                    };
                    item_handle.set_title(new_title).unwrap();
                }
                _ => {}
            }
        }
        _ => {}
    }
}
