#[must_use]
pub fn convert(bytes: &[u8]) -> Option<Vec<u8>> {
    let (header, decoded) = qoi::decode_to_vec(bytes).ok()?;
    let width = header.width;
    let height = header.height;
    let channels = match header.channels {
        qoi::Channels::Rgb => 3,
        qoi::Channels::Rgba => 4,
    };

    let width_usize = usize::try_from(width).ok()?;
    let height_usize = usize::try_from(height).ok()?;
    let width_i32 = i32::try_from(width).ok()?;
    let neg_height_i32 = i32::try_from(height).ok()?.checked_neg()?;

    let row_bytes = width_usize.checked_mul(3)?;
    let padding = (4 - (row_bytes % 4)) % 4;
    let image_size = (row_bytes + padding).checked_mul(height_usize)?;
    let file_size = image_size.checked_add(54)?;

    let file_size_u32 = u32::try_from(file_size).ok()?;
    let image_size_u32 = u32::try_from(image_size).ok()?;

    let mut bmp = Vec::with_capacity(file_size);

    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size_u32.to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&54u32.to_le_bytes());

    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&width_i32.to_le_bytes());
    bmp.extend_from_slice(&neg_height_i32.to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&image_size_u32.to_le_bytes());
    bmp.extend_from_slice(&[0; 16]);

    let pad_bytes = [0u8; 3];
    let row_stride = width_usize.checked_mul(channels)?;
    for y in 0..height_usize {
        let src_row_start = y.checked_mul(row_stride)?;
        let src_row = &decoded[src_row_start..src_row_start + row_stride];
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
