//! Unread badge on the tray icon.
//!
//! The Cinny frontend signals unread state only by swapping its favicon
//! (see `FaviconUpdater` in the cinny repo). `INIT_SCRIPT` watches that
//! favicon and reports changes to the `set_unread` command, which draws a red
//! dot onto the tray icon.

use tauri::{image::Image, AppHandle};

/// Injected into the main webview. The unread/highlight favicons are inlined
/// as data URIs in release builds, so they are recognised by their colours
/// (#989898 / #45B83B) as well as by file name in dev builds.
pub const INIT_SCRIPT: &str = r#"
(function () {
  var last = null;
  function sync(el) {
    var href = el.getAttribute('href') || '';
    var unread = /unread|highlight|989898|45b83b/i.test(href);
    if (unread === last) return;
    last = unread;
    window.__TAURI_INTERNALS__.invoke('set_unread', { unread: unread });
  }
  document.addEventListener('DOMContentLoaded', function () {
    var el = document.getElementById('favicon');
    if (!el) return;
    new MutationObserver(function () { sync(el); })
      .observe(el, { attributes: true, attributeFilter: ['href'] });
    sync(el);
  });
})();
"#;

/// Return a copy of `icon` with a red dot in the top-right corner.
pub fn with_dot(icon: &Image<'_>) -> Image<'static> {
    let (w, h) = (icon.width() as i32, icon.height() as i32);
    let mut rgba = icon.rgba().to_vec();

    let r = w.min(h) as f32 * 0.22;
    let border = (r * 0.18).max(1.0);
    let (cx, cy) = (w as f32 - r - border, r + border);

    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            let color = if d <= r {
                [0xE5, 0x39, 0x35, 0xFF]
            } else if d <= r + border {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                continue;
            };
            let i = ((y * w + x) * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&color);
        }
    }

    Image::new_owned(rgba, w as u32, h as u32)
}

#[tauri::command]
pub fn set_unread(app: AppHandle, unread: bool) {
    #[cfg(not(target_os = "linux"))]
    crate::tray::set_unread(&app, unread);
    #[cfg(target_os = "linux")]
    crate::tray_linux::set_unread(&app, unread);
}
