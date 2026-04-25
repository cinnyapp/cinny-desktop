use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, RgbaImage};
use tauri;

#[tauri::command]
pub fn paste_file() -> Result<Option<String>, String> {
    let mut clipboard = Clipboard::new().map_err(|err| err.to_string())?;
    let image_data = if let Ok(a) = clipboard.get_image() {
        a
    } else {
        return Ok(None);
    };
    let image: RgbaImage = ImageBuffer::from_raw(
        image_data.width as u32,
        image_data.height as u32,
        image_data.bytes.into_owned(),
    )
    .ok_or("failed to convert image".to_string())?;

    let mut buffer = vec![];
    image
        .write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageOutputFormat::Png,
        )
        .map_err(|err| err.to_string())?;

    let base64_str = general_purpose::STANDARD_NO_PAD.encode(buffer);
    Ok(Some(base64_str))
}
