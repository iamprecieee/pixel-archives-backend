use std::io::Cursor;

use png::{BitDepth, ColorType, Encoder};

use crate::{
    error::{AppError, Result},
    infrastructure::db::entities::pixel::Model as Pixel,
};

pub fn generate_png(pixels: &[Pixel]) -> Result<Vec<u8>> {
    let mut canvas_data = vec![(255u8, 255u8, 255u8); 1024];

    for pixel in pixels {
        let index = (pixel.y as usize) * 32 + (pixel.x as usize);
        if index < 1024 {
            canvas_data[index] = convert_color_index_to_rgb(pixel.color as u8);
        }
    }

    // Scales 16x (512x512).
    let scale = 16u32;
    let width = 32 * scale;
    let height = 32 * scale;

    let mut scaled_data = Vec::with_capacity((width * height) as usize * 3);
    for y in 0..height {
        for x in 0..width {
            let src_x = (x / scale) as usize;
            let src_y = (y / scale) as usize;
            let (r, g, b) = canvas_data[src_y * 32 + src_x];
            scaled_data.push(r);
            scaled_data.push(g);
            scaled_data.push(b);
        }
    }

    let mut png_data = Vec::new();
    {
        let mut encoder = Encoder::new(Cursor::new(&mut png_data), width, height);
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| AppError::InternalServerError(format!("PNG header error: {}", e)))?;
        writer
            .write_image_data(&scaled_data)
            .map_err(|e| AppError::InternalServerError(format!("PNG write error: {}", e)))?;
    }

    Ok(png_data)
}

pub fn generate_png_from_colors(pixel_colors: &[u8]) -> Result<Vec<u8>> {
    let mut canvas_data = vec![(255u8, 255u8, 255u8); 1024];

    // Unpack 6-bit colors: 4 pixels/3 bytes
    for group in 0..256 {
        let base_byte = group * 3;
        let base_pixel = group * 4;

        if base_byte + 2 < pixel_colors.len() {
            let b0 = pixel_colors[base_byte];
            let b1 = pixel_colors[base_byte + 1];
            let b2 = pixel_colors[base_byte + 2];

            // Unpack: c0 = b0[7:2], c1 = b0[1:0]b1[7:4], c2 = b1[3:0]b2[7:6], c3 = b2[5:0]
            let c0 = (b0 >> 2) & 0x3F;
            let c1 = ((b0 & 0x03) << 4) | ((b1 >> 4) & 0x0F);
            let c2 = ((b1 & 0x0F) << 2) | ((b2 >> 6) & 0x03);
            let c3 = b2 & 0x3F;

            if base_pixel < 1024 {
                canvas_data[base_pixel] = convert_color_index_to_rgb(c0);
            }
            if base_pixel + 1 < 1024 {
                canvas_data[base_pixel + 1] = convert_color_index_to_rgb(c1);
            }
            if base_pixel + 2 < 1024 {
                canvas_data[base_pixel + 2] = convert_color_index_to_rgb(c2);
            }
            if base_pixel + 3 < 1024 {
                canvas_data[base_pixel + 3] = convert_color_index_to_rgb(c3);
            }
        }
    }

    // 16x scale for better visibility (512x512 output)
    let scale = 16u32;
    let width = 32 * scale;
    let height = 32 * scale;

    let mut scaled_data = Vec::with_capacity((width * height) as usize * 3);
    for y in 0..height {
        for x in 0..width {
            let src_x = (x / scale) as usize;
            let src_y = (y / scale) as usize;
            let (r, g, b) = canvas_data[src_y * 32 + src_x];
            scaled_data.push(r);
            scaled_data.push(g);
            scaled_data.push(b);
        }
    }

    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(Cursor::new(&mut png_data), width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| AppError::InternalServerError(format!("PNG header error: {}", e)))?;
        writer
            .write_image_data(&scaled_data)
            .map_err(|e| AppError::InternalServerError(format!("PNG write error: {}", e)))?;
    }

    Ok(png_data)
}

fn convert_color_index_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        // Row 1: Grayscale
        0 => (0x00, 0x00, 0x00), // #000000
        1 => (0x1a, 0x1a, 0x1a), // #1a1a1a
        2 => (0x33, 0x33, 0x33), // #333333
        3 => (0x4d, 0x4d, 0x4d), // #4d4d4d
        4 => (0x66, 0x66, 0x66), // #666666
        5 => (0x80, 0x80, 0x80), // #808080
        6 => (0x99, 0x99, 0x99), // #999999
        7 => (0xb3, 0xb3, 0xb3), // #b3b3b3
        // Row 2: More grayscale + basics
        8 => (0xcc, 0xcc, 0xcc),  // #cccccc
        9 => (0xe6, 0xe6, 0xe6),  // #e6e6e6
        10 => (0xff, 0xff, 0xff), // #ffffff
        11 => (0xa9, 0x38, 0x38), // #A93838
        12 => (0xf5, 0xf5, 0xdc), // #F5F5DC
        13 => (0x8b, 0x00, 0x00), // #8B0000
        14 => (0xdc, 0x14, 0x3c), // #DC143C
        15 => (0xff, 0x63, 0x47), // #FF6347
        // Row 3: Reds to Oranges
        16 => (0xff, 0x45, 0x00), // #FF4500
        17 => (0xff, 0x8c, 0x00), // #FF8C00
        18 => (0xff, 0xa5, 0x00), // #FFA500
        19 => (0xff, 0xd7, 0x00), // #FFD700
        20 => (0xff, 0xff, 0x00), // #FFFF00
        21 => (0xad, 0xff, 0x2f), // #ADFF2F
        22 => (0x7f, 0xff, 0x00), // #7FFF00
        23 => (0x00, 0xff, 0x00), // #00FF00 (Green!)
        // Row 4: Greens
        24 => (0x32, 0xcd, 0x32), // #32CD32
        25 => (0x22, 0x8b, 0x22), // #228B22
        26 => (0x00, 0x64, 0x00), // #006400
        27 => (0x00, 0x8b, 0x8b), // #008B8B
        28 => (0x20, 0xb2, 0xaa), // #20B2AA
        29 => (0x00, 0xce, 0xd1), // #00CED1
        30 => (0x00, 0xff, 0xff), // #00FFFF
        31 => (0x00, 0xbf, 0xff), // #00BFFF
        // Row 5: Blues
        32 => (0x1e, 0x90, 0xff), // #1E90FF
        33 => (0x00, 0x00, 0xff), // #0000FF (Blue!)
        34 => (0x00, 0x00, 0xcd), // #0000CD
        35 => (0x00, 0x00, 0x8b), // #00008B
        36 => (0x19, 0x19, 0x70), // #191970
        37 => (0x4b, 0x00, 0x82), // #4B0082
        38 => (0x8b, 0x00, 0x8b), // #8B008B
        39 => (0x94, 0x00, 0xd3), // #9400D3
        // Row 6: Purples to Pinks
        40 => (0x99, 0x32, 0xcc), // #9932CC
        41 => (0xba, 0x55, 0xd3), // #BA55D3
        42 => (0xda, 0x70, 0xd6), // #DA70D6
        43 => (0xff, 0x00, 0xff), // #FF00FF
        44 => (0xff, 0x69, 0xb4), // #FF69B4
        45 => (0xff, 0x14, 0x93), // #FF1493
        46 => (0xc7, 0x15, 0x85), // #C71585
        47 => (0xdb, 0x70, 0x93), // #DB7093
        // Row 7: Browns and Earth tones
        48 => (0x8b, 0x45, 0x13), // #8B4513
        49 => (0xa0, 0x52, 0x2d), // #A0522D
        50 => (0xd2, 0x69, 0x1e), // #D2691E
        51 => (0xcd, 0x85, 0x3f), // #CD853F
        52 => (0xde, 0xb8, 0x87), // #DEB887
        53 => (0xf5, 0xde, 0xb3), // #F5DEB3
        54 => (0xfa, 0xeb, 0xd7), // #FAEBD7
        55 => (0xff, 0xe4, 0xc4), // #FFE4C4
        // Row 8: More earth + pastels
        56 => (0xff, 0xda, 0xb9), // #FFDAB9
        57 => (0xff, 0xe4, 0xe1), // #FFE4E1
        58 => (0xff, 0xf0, 0xf5), // #FFF0F5
        59 => (0xe6, 0xe6, 0xfa), // #E6E6FA
        60 => (0xd8, 0xbf, 0xd8), // #D8BFD8
        61 => (0xdd, 0xa0, 0xdd), // #DDA0DD
        62 => (0xee, 0x82, 0xee), // #EE82EE
        63 => (0xff, 0xff, 0xe0), // #FFFFE0
        _ => (0x80, 0x80, 0x80),  // Fallback gray
    }
}
