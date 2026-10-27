#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// mod menu;

use tauri::{webview::{NewWindowResponse, WebviewWindowBuilder}, WebviewUrl, TitleBarStyle};
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
    #[cfg(feature = "updater")]
    let mut builder = tauri::Builder::default();
    #[cfg(not(feature = "updater"))]
    let builder = tauri::Builder::default();

    // #[cfg(target_os = "macos")]
    // {
    //     builder = builder.menu(menu::menu());
    // }

    #[cfg(feature = "updater")]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_localhost::Builder::new(port).build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
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

            let init_script = r#"
                if (window.__TAURI_INTERNALS__) {
                    class TauriNotification {
                        constructor(title, options) {
                            this.title = title;
                            this.options = options || {};
                            window.__TAURI_INTERNALS__.invoke('plugin:notification|notify', {
                                options: {
                                    title: this.title,
                                    body: this.options.body || '',
                                    icon: this.options.icon || '',
                                }
                            }).catch(console.error);
                        }
                        static get permission() {
                            return window.__tauriNotificationPermission || 'default';
                        }
                        static requestPermission() {
                            return window.__TAURI_INTERNALS__.invoke('plugin:notification|request_permission')
                                .then(function(permission) {
                                    window.__tauriNotificationPermission = permission;
                                    if (window.__tauriNotificationPermissionStatus) {
                                        window.__tauriNotificationPermissionStatus.state = permission;
                                        if (typeof window.__tauriNotificationPermissionStatus.onchange === 'function') {
                                            window.__tauriNotificationPermissionStatus.onchange.call(window.__tauriNotificationPermissionStatus);
                                        }
                                    }
                                    return permission;
                                })
                                .catch(function() { return 'denied'; });
                        }
                        close() {}
                    }
                    window.Notification = TauriNotification;

                    const originalQuery = navigator.permissions.query;
                    navigator.permissions.query = function(parameters) {
                        if (parameters && parameters.name === 'notifications') {
                            return window.__TAURI_INTERNALS__.invoke('plugin:notification|is_permission_granted')
                                .then(function(isGranted) {
                                    const state = isGranted ? 'granted' : (window.__tauriNotificationPermission === 'denied' ? 'denied' : 'prompt');
                                    window.__tauriNotificationPermission = state;
                                    
                                    if (!window.__tauriNotificationPermissionStatus) {
                                        window.__tauriNotificationPermissionStatus = {
                                            name: 'notifications',
                                            state: state,
                                            onchange: null,
                                            addEventListener: function(type, listener) {
                                                if (type === 'change') this.onchange = listener;
                                            },
                                            removeEventListener: function(type, listener) {
                                                if (type === 'change' && this.onchange === listener) this.onchange = null;
                                            }
                                        };
                                    } else {
                                        window.__tauriNotificationPermissionStatus.state = state;
                                    }
                                    return window.__tauriNotificationPermissionStatus;
                                });
                        }
                        return originalQuery.call(navigator.permissions, parameters);
                    };
                }
            "#;

            let app_handle = app.handle().clone();
            let window_builder = WebviewWindowBuilder::new(app, "main".to_string(), window_url)
                .title("Cinny")
                .initialization_script(init_script)
                .disable_drag_drop_handler()
                .on_new_window(move |url, _features| {
                    let _ = app_handle.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                });

            #[cfg(target_os = "macos")]
            let window_builder = window_builder.title_bar_style(TitleBarStyle::Transparent);
            
            window_builder.build()?;
            Ok(())
        })
        .run(context)
        .expect("error while building tauri application");
}
