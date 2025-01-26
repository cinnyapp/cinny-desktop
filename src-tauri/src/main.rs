#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(target_os = "macos")]
mod menu;

//use tauri::{utils::config::App, WebviewUrl};
use tauri::WebviewUrl;

fn main() {
    let port = 44548;

    let mut context = tauri::generate_context!();
    let url = format!("http://localhost:{}", port).parse().unwrap();
    let window_url = WebviewUrl::External(url);
    // rewrite the config so the IPC is enabled on this URL
    //context.config_mut().build.dist_dir = AppUrl::Url(window_url.clone());
    context.config_mut().build.frontend_dist = AppUrl::Url(window_url.clone());
    //context.config_mut().build.dev_path = AppUrl::Url(window_url.clone());
    context.config_mut().build.dev_url = AppUrl::Url(window_url.clone());
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init());

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .run(context)
        .expect("error while building tauri application")
}
