//! Color types and HSV → RGB conversion.
//!
//! The HSV convention uses `h` sweeping a full turn over 0..=255
//! (so each 60° sector is 43 wide), `s`/`v` are ordinary 0..=255 fractions.

/// 8-bit HSV triple.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Hsv {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

impl Hsv {
    pub const fn new(h: u8, s: u8, v: u8) -> Self {
        Self { h, s, v }
    }
}

/// 8-bit RGB triple.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Integer HSV → RGB using the six-sector spectrum (same shape as the
/// `smart-leds` implementation). `h` partitions into 43-wide sectors; inside
/// each sector one channel holds at `v`, one at `p`, and the third ramps.
pub fn hsv_to_rgb(hsv: Hsv) -> Rgb {
    let h = hsv.h as u32;
    let s = hsv.s as u32;
    let v = hsv.v as u32;

    let sector = h / 43;
    let remainder = (h - sector * 43) * 6; // 0..=240 inside each sector

    let p = (v * (255 - s)) >> 8;
    let q = (v * (255 - ((s * remainder) >> 8))) >> 8;
    let t = (v * (255 - ((s * (255 - remainder)) >> 8))) >> 8;

    match sector {
        0 => Rgb::new(v as u8, t as u8, p as u8),
        1 => Rgb::new(q as u8, v as u8, p as u8),
        2 => Rgb::new(p as u8, v as u8, t as u8),
        3 => Rgb::new(p as u8, q as u8, v as u8),
        4 => Rgb::new(t as u8, p as u8, v as u8),
        _ => Rgb::new(v as u8, p as u8, q as u8),
    }
}
