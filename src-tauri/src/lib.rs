#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;

use tauri::{webview::WebviewWindowBuilder, WebviewUrl};

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
        .setup(move |app| {
            // Use predefined config
            let mut window_config = app.config().app.windows.iter().find(|c| c.label == "main").unwrap().clone();

            // Dev: use devUrl from tauri.conf.json (http://localhost:8080) to support HMR
            #[cfg(dev)]
            let window_url = WebviewUrl::App(Default::default());

            // Release: tauri-plugin-localhost serves bundled frontend assets on this port
            // Default would be http://tauri.localhost or tauri://localhost depending on platform
            #[cfg(not(dev))]
            let window_url = {
                let url = format!("http://localhost:{}", port).parse().unwrap();
                WebviewUrl::External(url)
            };

            window_config.url = window_url;

            WebviewWindowBuilder::from_config(app, &window_config)?.build()?;
            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
