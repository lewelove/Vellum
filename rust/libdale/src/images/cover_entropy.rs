use image::DynamicImage;

#[must_use] 
pub fn calculate_entropy(img: &DynamicImage) -> usize {
    let gray = img.grayscale();
    let mut buf = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buf);
    let _ = gray.write_to(&mut cursor, image::ImageFormat::Png);
    buf.len()
}
