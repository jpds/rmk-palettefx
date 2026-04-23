//! 16-stop palette data and color interpolation.
//!
//! Each palette is 16 `u16` entries packing HSV as
//! `[val:4 | sat:4 | hue:8]`, matching the PaletteFx on-device format.
//! Saturation and value unpack via multiplication by 17 to recover the full
//! 0..=255 range.
//!
//! The palette constants are generated at build time from
//! `palettes/*.toml` (see `build.rs`). Edit a palette by changing its
//! TOML file; add a new one by dropping in another `.toml`.

use crate::color::Hsv;
use crate::math::{lerp8by8, scale8};

/// A 16-stop gradient palette in packed HSV16 format.
pub type Palette = [u16; 16];

/// Pack an HSV triple into the compact on-device format. Saturation and
/// value are stored in 4 bits each; the unpack step multiplies by 17 to
/// bring them back to 0..=255.
pub const fn hsv16(h: u8, s: u8, v: u8) -> u16 {
    (((v as u16) >> 4) << 12) | (((s as u16) >> 4) << 8) | (h as u16 & 0xff)
}

#[inline]
pub const fn unpack_hsv16(word: u16) -> Hsv {
    Hsv {
        h: word as u8,
        s: (((word >> 8) as u8) & 0x0f) * 17,
        v: (((word >> 12) as u8) & 0x0f) * 17,
    }
}

/// Linearly interpolate a palette at position `x` in 0..=255, scaling the
/// result's saturation and value by the supplied global multipliers.
///
/// `sat_scale`/`val_scale` control saturation and value scaling;
/// pass 255 for "unmodified" output.
pub fn interp_color(palette: &Palette, x: u8, sat_scale: u8, val_scale: u8) -> Hsv {
    // Clamp to [8, 247] and remap to [0, 239] so the endpoint stops stay
    // within the 15 interpolation segments.
    let x = if x <= 8 {
        0
    } else if x < 247 {
        x - 8
    } else {
        239
    };

    let i = (x >> 4) as usize;
    let frac = x << 4;

    let a = unpack_hsv16(palette[i]);
    let b = unpack_hsv16(palette[i + 1]);

    // If the two stops span the 0/255 hue seam, XOR both hues with 128 so
    // the linear interpolation takes the short way round the colour wheel.
    let hue_wrap = 128 & a.h.abs_diff(b.h);

    Hsv {
        h: lerp8by8(a.h ^ hue_wrap, b.h ^ hue_wrap, frac) ^ hue_wrap,
        s: scale8(lerp8by8(a.s, b.s, frac), sat_scale),
        v: scale8(lerp8by8(a.v, b.v, frac), val_scale),
    }
}

include!(concat!(env!("OUT_DIR"), "/palettes_generated.rs"));
