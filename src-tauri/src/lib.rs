#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;

use tauri::{
    webview::{NewWindowResponse, WebviewWindowBuilder},
    WebviewUrl,
};
use tauri_plugin_opener::OpenerExt;

const REQUEST_NOTIFICATION_PERMISSION_JS: &str = r#"
  window.addEventListener('DOMContentLoaded', () => {
    try {
      if (window.Notification && typeof window.Notification.requestPermission === 'function') {
        window.Notification.requestPermission().catch(() => {});
      }
    } catch (e) { console.log(e) }
  });
  "#;

pub fn run() {
    let port: u16 = 44548;
    let context = tauri::generate_context!();
    let builder = tauri::Builder::default();

    // #[cfg(target_os = "macos")]
    // {
    //     builder = builder.menu(menu::menu());
    // }

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            // Dev: use devUrl from tauri.conf.json (http://localhost:8080) to support HMR
            #[cfg(debug_assertions)]
            let window_url = WebviewUrl::App(Default::default());

            // Release: tauri-plugin-localhost serves bundled frontend assets on this port
            #[cfg(not(debug_assertions))]
            let window_url = {
                let url = format!("http://localhost:{}", port).parse().unwrap();
                WebviewUrl::External(url)
            };

            let app_handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                .title("Cinny")
                .disable_drag_drop_handler()
                .initialization_script(REQUEST_NOTIFICATION_PERMISSION_JS)
                .on_new_window(move |url, _features| {
                    let _ = app_handle.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                })
                .build()?;
            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
