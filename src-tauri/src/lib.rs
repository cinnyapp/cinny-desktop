#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;
mod badge;
#[cfg(not(target_os = "linux"))]
mod tray;
#[cfg(target_os = "linux")]
mod tray_linux;

use tauri::{webview::{NewWindowResponse, WebviewWindowBuilder}, Manager, WebviewUrl, TitleBarStyle};
use tauri_plugin_opener::OpenerExt;

#[cfg(feature = "updater")]
use tauri_plugin_updater::UpdaterExt;
#[cfg(feature = "updater")]
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub fn run() {
    
    for key in ["NO_PROXY", "no_proxy"] {
        let current_val = std::env::var(key).unwrap_or_default();
        if !current_val.contains("localhost") {
            let new_val = if current_val.is_empty() {
                "localhost,127.0.0.1".to_string()
            } else {
                format!("{},localhost,127.0.0.1", current_val)
            };
            std::env::set_var(key, new_val);
        }
    }

    let port: u16 = 44548;
    let context = tauri::generate_context!();
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    // #[cfg(target_os = "macos")]
    // {
    //     builder = builder.menu(menu::menu());
    // }

    // Must be registered first: a second launch focuses the running instance
    // (which may be hidden in the tray) instead of opening another window.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    #[cfg(feature = "updater")]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![badge::set_unread])
        .setup(move |app| {
            #[cfg(feature = "updater")]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let updater = match handle.updater() {
                        Ok(u) => u,
                        Err(e) => {
                            eprintln!("Updater not available: {}", e);
                            return;
                        }
                    };
                    if let Ok(Some(update)) = updater.check().await {
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
            }

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
            let window_builder = WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                .title("Cinny")
                .initialization_script(badge::INIT_SCRIPT)
                .disable_drag_drop_handler()
                .on_new_window(move |url, _features| {
                    let _ = app_handle.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                });

            #[cfg(target_os = "macos")]
            let window_builder = window_builder.title_bar_style(TitleBarStyle::Transparent);
            
            let window = window_builder.build()?;

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
