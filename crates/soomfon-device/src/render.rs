//! Tiny helpers for building key images.
//!
//! Stream-deck keys are only 60x60, so labels and solid fills cover most of
//! what the app needs before it loads real artwork. Text is drawn with the
//! public-domain 8x8 bitmap font (`font8x8`), scaled up by an integer factor so
//! glyphs stay crisp instead of blurring on the small panel.

use font8x8::{UnicodeFonts, BASIC_FONTS};
use image::{Rgb, RgbImage};

/// Side length of one font cell, in source pixels.
const GLYPH: u32 = 8;
/// Fraction of the canvas the text block may occupy before it stops scaling up.
const FILL: f32 = 0.85;

/// Build a solid-colour image of the given `(width, height)`.
pub fn solid(size: (u32, u32), color: [u8; 3]) -> RgbImage {
    RgbImage::from_pixel(size.0, size.1, Rgb(color))
}

/// Render a short label centred on a solid background.
///
/// `text` may contain newlines to lay out multiple lines. The font is scaled by
/// the largest integer factor that lets the longest line fit, so a one-letter
/// label fills the key while a longer one shrinks to fit.
pub fn text(size: (u32, u32), text: &str, fg: [u8; 3], bg: [u8; 3]) -> RgbImage {
    let mut img = solid(size, bg);

    let lines: Vec<&str> = text.lines().collect();
    let cols = lines
        .iter()
        .map(|line| line.chars().count() as u32)
        .max()
        .unwrap_or(0);
    let rows = lines.len() as u32;
    if cols == 0 || rows == 0 {
        return img;
    }

    let fit_w = (size.0 as f32 * FILL) / (cols * GLYPH) as f32;
    let fit_h = (size.1 as f32 * FILL) / (rows * GLYPH) as f32;
    let scale = fit_w.min(fit_h).floor().max(1.0) as u32;

    let block_w = cols * GLYPH * scale;
    let block_h = rows * GLYPH * scale;
    let block_x = (size.0.saturating_sub(block_w)) / 2;
    let block_y = (size.1.saturating_sub(block_h)) / 2;

    let fg = Rgb(fg);
    for (row, line) in lines.iter().enumerate() {
        let line_w = line.chars().count() as u32 * GLYPH * scale;
        let line_x = block_x + (block_w.saturating_sub(line_w)) / 2;
        let line_y = block_y + row as u32 * GLYPH * scale;

        for (col, ch) in line.chars().enumerate() {
            let glyph = BASIC_FONTS
                .get(ch)
                .or_else(|| BASIC_FONTS.get('?'))
                .unwrap_or([0; 8]);
            blit_glyph(
                &mut img,
                glyph,
                line_x + col as u32 * GLYPH * scale,
                line_y,
                scale,
                fg,
            );
        }
    }

    img
}

/// Draw one 8x8 glyph, scaled, with its top-left corner at `(x, y)`.
///
/// In `font8x8` each byte is one row (top first) and bit `n` (from the LSB) is
/// the pixel at column `n` (left first).
fn blit_glyph(img: &mut RgbImage, glyph: [u8; 8], x: u32, y: u32, scale: u32, color: Rgb<u8>) {
    let (w, h) = img.dimensions();

    for (gy, row_bits) in glyph.iter().enumerate() {
        for gx in 0..GLYPH {
            if row_bits & (1 << gx) == 0 {
                continue;
            }

            let px0 = x + gx * scale;
            let py0 = y + gy as u32 * scale;
            for dy in 0..scale {
                for dx in 0..scale {
                    let (px, py) = (px0 + dx, py0 + dy);
                    if px < w && py < h {
                        img.put_pixel(px, py, color);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_fills_every_pixel() {
        let img = solid((60, 60), [10, 20, 30]);
        assert_eq!(img.dimensions(), (60, 60));
        assert_eq!(*img.get_pixel(0, 0), Rgb([10, 20, 30]));
        assert_eq!(*img.get_pixel(59, 59), Rgb([10, 20, 30]));
    }

    #[test]
    fn text_keeps_canvas_size_and_draws_foreground() {
        let img = text((60, 60), "5", [255, 255, 255], [0, 0, 0]);
        assert_eq!(img.dimensions(), (60, 60));
        let drawn = img.pixels().filter(|p| **p == Rgb([255, 255, 255])).count();
        assert!(drawn > 0, "a glyph should leave foreground pixels");
    }

    #[test]
    fn empty_text_is_pure_background() {
        let img = text((60, 60), "", [255, 255, 255], [1, 2, 3]);
        assert!(img.pixels().all(|p| *p == Rgb([1, 2, 3])));
    }

    #[test]
    fn space_draws_nothing() {
        let img = text((60, 60), " ", [255, 255, 255], [0, 0, 0]);
        let drawn = img.pixels().filter(|p| **p == Rgb([255, 255, 255])).count();
        assert_eq!(drawn, 0);
    }
}
