use std::io::Cursor;

use egui::IconData;
use ico::IconDir;

pub fn generate_icon_data() -> IconData {
    let ico_bytes = include_bytes!("../assets/icon.ico");
    let icon_dir = IconDir::read(Cursor::new(ico_bytes)).expect("Failed to parse ico format");
    let entry = icon_dir
        .entries()
        .first()
        .expect("The ico file contains no images");
    let image = entry.decode().expect("Failed to decode ICO image entry");
    let width = image.width();
    let height = image.height();
    let rgba = image.rgba_data().to_vec();
    IconData {
        rgba,
        width,
        height,
    }
}
