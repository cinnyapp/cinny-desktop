#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#[cfg(target_os = "macos")]
mod menu;
mod tray;

fn main() {
    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    let builder = builder
        .system_tray(tray::system_tray())
        .on_system_tray_event(tray::system_tray_handler);

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(run_event_handler)
}

fn run_event_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::WindowEvent { label, event, .. } => {
            tray::window_event_handler(app, &label, &event);
        }
        _ => {}
    }
}
