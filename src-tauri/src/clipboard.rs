use arboard::{Clipboard};
use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, RgbaImage};
use std::fs::File;
use std::io::Read;
use tauri;

#[tauri::command]
pub fn clipboard_read_image() -> Result<String, String> {
    let mut clipboard = Clipboard::new().unwrap();
    let image = clipboard.get_image().map_err(|err| err.to_string())?;
    let tmp_dir = tempfile::Builder::new()
        .prefix("clipboard-img")
        .tempdir()
        .map_err(|err| err.to_string())?;

    let fname = tmp_dir.path().join("clipboard-img.png");

    let image2: RgbaImage = ImageBuffer::from_raw(
        image.width.try_into().unwrap(),
        image.height.try_into().unwrap(),
        image.bytes.into_owned(),
    ).unwrap();

    image2.save(fname.clone()).map_err(|err| err.to_string())?;

    let mut file = File::open(fname.clone()).unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();
    let base64_str = general_purpose::STANDARD_NO_PAD.encode(buffer);

    Ok(base64_str)
}
