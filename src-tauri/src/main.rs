#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(target_os = "macos")]
mod menu;

use tauri::{utils::config::AppUrl, WindowUrl};

fn main() {
    let port = 44548;

    let mut context = tauri::generate_context!();

    // macOS "App Nap" periodically pauses our app when it's in the background.
    // We need to prevent that so our intervals are not interrupted.
    #[cfg(target_os = "macos")]
    macos_app_nap::prevent();

    let url = format!("http://localhost:{}", port).parse().unwrap();
    let window_url = WindowUrl::External(url);
    // rewrite the config so the IPC is enabled on this URL
    context.config_mut().build.dist_dir = AppUrl::Url(window_url.clone());
    context.config_mut().build.dev_path = AppUrl::Url(window_url.clone());
    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(context)
        .expect("error while building tauri application")
}
