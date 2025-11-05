#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(target_os = "macos")]
mod menu;

fn main() {
    let port = 44548;

    let context = tauri::generate_context!();

    let builder = tauri::Builder::default();

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_menu(menu::menu(app.handle())?)?;

            Ok(())
        })
        .run(context)
        .expect("error while building tauri application")
}
