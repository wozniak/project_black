// drawing

use crate::{WIDTH, HEIGHT};

pub fn draw_string(buffer: &mut Vec<u32>, font: &bdf::Font, x: u32, y: u32, color: u32, text: &str) {
    let glyphs = font.glyphs();
    let mut offset_x = 0u32;
    let mut offset_y = 0u32;
    for ch in text.chars() {
        if ch == '\n' {
            offset_y += (glyphs.get(&'a').unwrap().height() as f32 * 1.2) as u32;
            offset_x = 0;
            continue
        }

        let glyph = glyphs.get(&ch).unwrap();

        for pixel in glyph.pixels() {
            if pixel.1 { set_pixel(buffer, x + offset_x + pixel.0.0, y + offset_y + pixel.0.1, color) };
        }
        offset_x += glyph.width();
    }
}

pub fn rectangle(buffer: &mut Vec<u32>, top_left: (u32, u32), bottom_right: (u32, u32), filled: bool, color: u32) {
    if filled {
        for x in top_left.0..bottom_right.0 {
            for y in top_left.1..bottom_right.1 {
                set_pixel(buffer, x, y, color);
            }
        }
    } else {
        for x in top_left.0..bottom_right.0 {
            for y in top_left.1..bottom_right.1 {
                if y == top_left.1 || y == bottom_right.1
                    || x == top_left.0 || x == bottom_right.0 {
                    set_pixel(buffer, x, y, color);
                }
            }
        }
    }
}

pub fn draw_key(buffer: &mut Vec<u32>, n: usize, height: u32, color: u32) {
    let key_areas = [
        [WIDTH / 2 - 160, WIDTH / 2 - 80],
        [WIDTH / 2 - 80, WIDTH / 2],
        [WIDTH / 2, WIDTH / 2 + 80],
        [WIDTH / 2 + 80, WIDTH / 2 + 160],
    ];

    rectangle(buffer,
              (key_areas[n][0] as u32, height.max(10) - 10),
              (key_areas[n][1] as u32, height.min(HEIGHT as u32)),
              true,
              color
    );
}

fn set_pixel(buffer: &mut Vec<u32>, x: u32, y: u32, color: u32) {
    (*buffer)[y as usize * WIDTH + x as usize] = color
}