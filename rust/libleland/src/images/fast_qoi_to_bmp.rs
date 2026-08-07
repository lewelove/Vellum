#[allow(clippy::cast_possible_wrap)]
pub fn convert(bytes: &[u8]) -> Option<Vec<u8>> {
    let (header, decoded) = qoi::decode_to_vec(bytes).ok()?;
    let width = header.width;
    let height = header.height;
    let channels = header.channels as usize;

    let row_bytes = (width * 3) as usize;
    let padding = (4 - (row_bytes % 4)) % 4;
    let image_size = (row_bytes + padding) * (height as usize);
    let file_size = 54 + image_size;

    let mut bmp = Vec::with_capacity(file_size);

    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&54u32.to_le_bytes());

    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(-(height as i32)).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&(image_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 16]);

    let pad_bytes = [0u8; 3];
    for y in 0..(height as usize) {
        let src_row_start = y * (width as usize) * channels;
        let src_row = &decoded[src_row_start..src_row_start + ((width as usize) * channels)];
        for chunk in src_row.chunks_exact(channels) {
            bmp.push(chunk[2]);
            bmp.push(chunk[1]);
            bmp.push(chunk[0]);
        }
        if padding > 0 {
            bmp.extend_from_slice(&pad_bytes[..padding]);
        }
    }

    Some(bmp)
}
