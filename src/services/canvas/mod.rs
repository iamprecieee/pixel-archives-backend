use crate::infrastructure::db::entities::pixel::Model as Pixel;

pub mod types;

pub mod collaboration;
pub mod lifecycle;

/// Packs a canvas of pixels into 768 bytes using 6-bit color encoding.
///
/// Solana instruction limit: 1232 bytes. Each 3-byte sequence encodes 4 pixels (4 * 6 bits = 24 bits).
/// Supports up to 64 colors (6 bits per pixel).
///
/// Layout of each 3-byte group:
/// Byte 0: [Pixel 0 (6 bits)] [Pixel 1 (hi 2 bits)]
/// Byte 1: [Pixel 1 (lo 4 bits)] [Pixel 2 (hi 4 bits)]
/// Byte 2: [Pixel 2 (lo 2 bits)] [Pixel 3 (6 bits)]
pub fn pack_pixels_to_colors(pixels: &[Pixel], width: u8, height: u8) -> [u8; 768] {
    const DEFAULT_COLOR: u8 = 10; // White
    const GROUPS: usize = 256; // 1024 pixels / 4 pixels per group

    let total_pixels = (width as usize) * (height as usize);

    // Flatten pixel array into color indices
    let mut colors = vec![DEFAULT_COLOR; total_pixels];
    for pixel in pixels {
        let index = (pixel.y as usize) * (width as usize) + (pixel.x as usize);
        if index < total_pixels {
            colors[index] = pixel.color as u8 & 0x3F; // 6-bit mask
        }
    }

    let mut packed = [0u8; 768];

    for group_index in 0..GROUPS {
        let base_pixel = group_index * 4;
        let base_byte = group_index * 3;

        let c0 = colors.get(base_pixel).copied().unwrap_or(DEFAULT_COLOR);
        let c1 = colors.get(base_pixel + 1).copied().unwrap_or(DEFAULT_COLOR);
        let c2 = colors.get(base_pixel + 2).copied().unwrap_or(DEFAULT_COLOR);
        let c3 = colors.get(base_pixel + 3).copied().unwrap_or(DEFAULT_COLOR);

        packed[base_byte] = (c0 << 2) | (c1 >> 4);
        packed[base_byte + 1] = ((c1 & 0x0F) << 4) | (c2 >> 2);
        packed[base_byte + 2] = ((c2 & 0x03) << 6) | c3;
    }

    packed
}

pub use collaboration::*;
pub use lifecycle::*;
