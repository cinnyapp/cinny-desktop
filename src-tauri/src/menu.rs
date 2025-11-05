use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};

pub(crate) fn menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let cinny_submenu = SubmenuBuilder::new(app, "Cinny")
        .item(&PredefinedMenuItem::about(app, Some("Cinny"), None)?)
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;

    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_submenu = SubmenuBuilder::new(app, "View")
        .item(&PredefinedMenuItem::fullscreen(app, None)?)
        .build()?;

    let window_submenu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .item(&PredefinedMenuItem::maximize(app, None)?)
        .build()?;

    MenuBuilder::new(app)
        .items(&[
            &cinny_submenu,
            &edit_submenu,
            &view_submenu,
            &window_submenu,
        ])
        .build()
}
