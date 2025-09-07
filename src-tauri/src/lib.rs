#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// #[cfg(target_os = "macos")]
// mod menu;

// use tauri::{webview::WebviewWindowBuilder, WebviewUrl};

// fn main() {
//     let port: u16 = 44548;

//     let mut context = tauri::generate_context!();
//     let url = format!("http://localhost:{}", port).parse().unwrap();
//     let window_url = WebviewUrl::External(url);
//     // context.config_mut().build.frontend_dist = WebviewWindowBuilder::Url(window_url.clone());
//     // context.config_mut().build.dev_url = WebviewWindowBuilder::Url(window_url.clone());

//     context.config_mut().build.frontend_dist = WebviewWindowBuilder::new(app, "Cinny", window_url.clone());
//     context.config_mut().build.dev_url = WebviewWindowBuilder::new(app, "Cinny", window_url.clone());
//     let builder = tauri::Builder::default();

//     #[cfg(target_os = "macos")]
//     let builder = builder.menu(menu::menu());

//     builder
//         .plugin(tauri_plugin_localhost::Builder::new(port).build())
//         .plugin(tauri_plugin_window_state::Builder::default().build())
//         .run(context)
//         .expect("error while building tauri application")
// }

use tauri::{webview::WebviewWindowBuilder, WebviewUrl};

pub fn run() {
    let port: u16 = 44548;
    let context = tauri::generate_context!();

    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    let builder = builder.menu(menu::menu());

    builder
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            let url = format!("http://localhost:{}", port).parse().unwrap();
            let window_url = WebviewUrl::External(url);

            WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                .title("Cinny")
                .build()?;

            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}

