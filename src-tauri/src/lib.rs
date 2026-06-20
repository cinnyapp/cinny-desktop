#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;
#[cfg(not(target_os = "linux"))]
mod tray;
#[cfg(target_os = "linux")]
mod tray_linux;

use tauri::{webview::{NewWindowResponse, WebviewWindowBuilder}, WebviewUrl};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub fn run() {
    let port: u16 = 44548;
    let context = tauri::generate_context!();
    let builder = tauri::Builder::default();

    // #[cfg(target_os = "macos")]
    // {
    //     builder = builder.menu(menu::menu());
    // }

    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(Some(update)) = handle.updater().unwrap().check().await {
                    let version = update.version.clone();
                    
                    let should_update = handle
                        .dialog()
                        .message(format!(
                            "Version {} is available.\n\nWould you like to update now?",
                            version
                        ))
                        .title("Update Available")
                        .kind(MessageDialogKind::Info)
                        .buttons(MessageDialogButtons::YesNo)
                        .blocking_show();

                    if should_update {
                        if update.download_and_install(|_, _| {}, || {}).await.is_ok() {
                            handle.restart();
                        }
                    }
                }
            });

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
            let window = WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                .title("Cinny")
                .disable_drag_drop_handler()
                .on_new_window(move |url, _features| {
                    let _ = app_handle.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                })
                .build()?;

            // Close to tray: hide the window instead of quitting the app.
            let win = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win.hide();
                }
            });

            #[cfg(not(target_os = "linux"))]
            tray::build(app.handle())?;
            #[cfg(target_os = "linux")]
            tray_linux::build(app.handle().clone());

            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
