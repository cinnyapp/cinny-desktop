#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;

use tauri::{self, webview::WebviewWindowBuilder, WebviewUrl};
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let port: u16 = 8080;
    let context = tauri::generate_context!();
    let builder = tauri::Builder::default();

    // #[cfg(target_os = "macos")]
    // {
    //     builder = builder.menu(menu::menu());
    // }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());
    };

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .setup(move |app| {
            let url = format!("http://localhost:{}", port).parse().unwrap();
            let window_url = WebviewUrl::External(url);
            let mut webview_builder =
                WebviewWindowBuilder::new(app, "cinny".to_string(), window_url);
            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                webview_builder = webview_builder.title("Cinny");
            };
            webview_builder.build()?;
            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
