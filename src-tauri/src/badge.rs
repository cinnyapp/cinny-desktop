//! Unread badge on the tray icon.
//!
//! The Cinny frontend signals unread state only by swapping its favicon
//! (see `FaviconUpdater` in the cinny repo). `INIT_SCRIPT` watches that
//! favicon and reports changes to the `set_unread` command, which draws a red
//! dot onto the tray icon.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{image::Image, AppHandle, Runtime};

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
    let (w, h) = (icon.width(), icon.height());
    let mut rgba = icon.rgba().to_vec();
    let r = w.min(h) as f32 * 0.22;
    let border = (r * 0.18).max(1.0);
    draw_dot(&mut rgba, w, h, w as f32 - r - border, r + border, r, border);
    Image::new_owned(rgba, w, h)
}

/// A red dot on a transparent square, used as the Windows taskbar overlay.
#[cfg(target_os = "windows")]
fn dot(size: u32) -> Image<'static> {
    let mut rgba = vec![0; (size * size * 4) as usize];
    let border = (size as f32 * 0.08).max(1.0);
    let c = size as f32 / 2.0;
    draw_dot(&mut rgba, size, size, c, c, c - border, border);
    Image::new_owned(rgba, size, size)
}

/// Paint a red dot with a white border, centred at (`cx`, `cy`).
fn draw_dot(rgba: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32, border: f32) {
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
}

/// Last unread state reported by the frontend.
static UNREAD: AtomicBool = AtomicBool::new(false);

/// Show or clear the unread indicator: a dot on the tray icon everywhere,
/// plus the Dock badge on macOS and a taskbar overlay on Windows.
#[tauri::command]
pub fn set_unread(app: AppHandle, unread: bool) {
    UNREAD.store(unread, Ordering::Relaxed);
    #[cfg(not(target_os = "linux"))]
    crate::tray::set_unread(&app, unread);
    #[cfg(target_os = "linux")]
    crate::tray_linux::set_unread(&app, unread);
    reapply(&app);
}

/// Apply the unread state to the Dock badge (macOS) or taskbar overlay
/// (Windows). The overlay needs a taskbar button, so this runs again
/// whenever the window is shown.
pub fn reapply<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Some(window) = tauri::Manager::get_webview_window(app, "main") {
        let unread = UNREAD.load(Ordering::Relaxed);
        #[cfg(target_os = "macos")]
        let _ = window.set_badge_label(unread.then(|| "\u{2022}".to_string()));
        #[cfg(target_os = "windows")]
        let _ = window.set_overlay_icon(unread.then(|| dot(32)));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = app;
}
