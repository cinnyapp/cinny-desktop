use tauri::{ GlobalShortcutManager, Manager };

//use crate::tray;

pub fn ready_event_handler<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>
) {
    let manager = app.global_shortcut_manager();
    //manager.clone().register("CmdOrCtrl+w", hide_closure(app)).unwrap();
    manager.clone().register("CmdOrCtrl+q", close_closure(app)).unwrap();
}

/*
fn hide_closure<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> impl Fn() {
    let window = app.get_window("main").unwrap();
    let app = app.clone();
    return move || {
        window.hide().unwrap();
        tray::toggle_window_state(window.clone(), app.tray_handle_by_id(tray::TRAY_LABEL).unwrap())
    }
}
*/

fn close_closure<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> impl Fn() {
    let window = app.get_window("main").unwrap();
    return move || window.close().unwrap();
}