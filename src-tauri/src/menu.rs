use tauri::menu::{MenuBuilder, SubmenuBuilder};
use tauri::AppHandle;

pub fn menu() -> tauri::menu::Menu {
    let app_menu = SubmenuBuilder::new(app, "Cinny")
        .about(Some(Default::default()))
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()
        .unwrap();

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()
        .unwrap();

    let view_menu = SubmenuBuilder::new(app, "View")
        .fullscreen() // `.fullscreen()` works instead of `.enter_fullscreen()`
        .build()
        .unwrap();

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .build() // no `.zoom()` method directly available
        .unwrap();

    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .build()
        .unwrap()
}
