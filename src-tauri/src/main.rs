#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(target_os = "macos")]
mod menu;
mod shortcuts;

fn main() {
  let builder = tauri::Builder::default();

  #[cfg(target_os = "macos")]
  let builder = builder.menu(menu::menu());

    builder
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(run_event_handler)
}

fn run_event_handler<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::Ready => {
            shortcuts::ready_event_handler(app);
        }
        _ => {}
    }
}
