//! Minimal PNG writer (no new dependencies — the crate's dep list
//! stays `mesher, proto, glam` as scaffolded). Writes 8-bit RGB PNGs
//! using zlib *stored* (uncompressed) blocks: larger files, but the
//! dumps are small evidence images, not shipped assets, and a stored
//! stream needs no compression library.

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(kind);
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Encode `rgb` (`width * height * 3` bytes, row-major, top row
/// first) as a PNG byte vector.
pub fn encode_png_rgb(width: u32, height: u32, rgb: &[u8]) -> Vec<u8> {
    assert_eq!(rgb.len(), (width * height * 3) as usize);
    // Raw image data with a filter byte (0 = none) per row.
    let mut raw = Vec::with_capacity((width * 3 + 1) as usize * height as usize);
    for y in 0..height {
        raw.push(0u8);
        let start = (y * width * 3) as usize;
        raw.extend_from_slice(&rgb[start..start + (width * 3) as usize]);
    }
    // zlib stream: header + stored blocks (<= 65535 bytes each) + adler.
    let mut zlib = vec![0x78u8, 0x01u8];
    let mut chunks = raw.chunks(65535).peekable();
    if raw.is_empty() {
        zlib.push(1);
        zlib.extend_from_slice(&0u16.to_le_bytes());
        zlib.extend_from_slice(&(!0u16).to_le_bytes());
    }
    while let Some(c) = chunks.next() {
        let last = chunks.peek().is_none();
        zlib.push(if last { 1 } else { 0 });
        zlib.extend_from_slice(&(c.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(c.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(c);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit, RGB, deflate, no interlace
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    png
}

pub fn write_png_rgb(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgb: &[u8],
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, encode_png_rgb(width, height, rgb))
}
