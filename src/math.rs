//! 8-bit fixed-point arithmetic helpers.
//!
//! All "fractional" quantities are u8 where 256 maps to 1.0. Angles for
//! [`sin8`]/[`cos8`] use the same convention: 256 is one full turn, output is
//! centred on 128.

use micromath::F32Ext;

/// `round((sin(2*pi*x/256) + 1) / 2 * 255)` for x in 0..=255.
#[rustfmt::skip]
const SIN8_LUT: [u8; 256] = [
    128, 131, 134, 137, 140, 143, 146, 149, 152, 155, 158, 162, 165, 167, 170, 173,
    176, 179, 182, 185, 188, 190, 193, 196, 198, 201, 203, 206, 208, 211, 213, 215,
    218, 220, 222, 224, 226, 228, 230, 232, 234, 235, 237, 238, 240, 241, 243, 244,
    245, 246, 248, 249, 250, 250, 251, 252, 253, 253, 254, 254, 254, 255, 255, 255,
    255, 255, 255, 255, 254, 254, 254, 253, 253, 252, 251, 250, 250, 249, 248, 246,
    245, 244, 243, 241, 240, 238, 237, 235, 234, 232, 230, 228, 226, 224, 222, 220,
    218, 215, 213, 211, 208, 206, 203, 201, 198, 196, 193, 190, 188, 185, 182, 179,
    176, 173, 170, 167, 165, 162, 158, 155, 152, 149, 146, 143, 140, 137, 134, 131,
    128, 124, 121, 118, 115, 112, 109, 106, 103, 100,  97,  93,  90,  88,  85,  82,
     79,  76,  73,  70,  67,  65,  62,  59,  57,  54,  52,  49,  47,  44,  42,  40,
     37,  35,  33,  31,  29,  27,  25,  23,  21,  20,  18,  17,  15,  14,  12,  11,
     10,   9,   7,   6,   5,   5,   4,   3,   2,   2,   1,   1,   1,   0,   0,   0,
      0,   0,   0,   0,   1,   1,   1,   2,   2,   3,   4,   5,   5,   6,   7,   9,
     10,  11,  12,  14,  15,  17,  18,  20,  21,  23,  25,  27,  29,  31,  33,  35,
     37,  40,  42,  44,  47,  49,  52,  54,  57,  59,  62,  65,  67,  70,  73,  76,
     79,  82,  85,  88,  90,  93,  97, 100, 103, 106, 109, 112, 115, 118, 121, 124,
];

/// 8-bit sine: `sin(2*pi*x/256)` scaled to `[0, 255]` with 128 at zero crossings.
#[inline]
pub fn sin8(x: u8) -> u8 {
    SIN8_LUT[x as usize]
}

/// 8-bit cosine: `cos(2*pi*x/256)` scaled to `[0, 255]`.
#[inline]
pub fn cos8(x: u8) -> u8 {
    SIN8_LUT[x.wrapping_add(64) as usize]
}

/// `atan2(y, x)` mapped into 0..=255 where 256 is one full turn.
pub fn atan2_8(y: i16, x: i16) -> u8 {
    if x == 0 && y == 0 {
        return 0;
    }
    let a = (y as f32).atan2(x as f32); // (-pi, pi]
    let turns = a * (1.0 / core::f32::consts::TAU);
    let wrapped = turns - F32Ext::floor(turns); // [0, 1)
    // Multiply first, then truncate. `as u8` saturates in Rust which is fine
    // since the floor-subtract keeps us safely below 1.0.
    (wrapped * 256.0) as u32 as u8
}

/// Integer square root: `floor(sqrt(x))` for x in 0..=65535. Result fits in u8.
pub fn sqrt16(x: u16) -> u8 {
    if x < 2 {
        return x as u8;
    }
    let mut res: u32 = 0;
    let mut bit: u32 = 1 << 14;
    let mut n: u32 = x as u32;
    while bit > n {
        bit >>= 2;
    }
    while bit != 0 {
        if n >= res + bit {
            n -= res + bit;
            res = (res >> 1) + bit;
        } else {
            res >>= 1;
        }
        bit >>= 2;
    }
    res as u8
}

/// `(i * scale) / 256`. u8 fixed-point multiply.
#[inline]
pub fn scale8(i: u8, scale: u8) -> u8 {
    (((i as u16) * (scale as u16)) >> 8) as u8
}

/// `(i * (1 + scale)) / 256`, u16 result. The `+1` ensures that
/// scaling full-range `i = 0xFFFF` by `scale = 0xFF` yields `0xFFFF`.
#[inline]
pub fn scale16by8(i: u16, scale: u8) -> u16 {
    (((i as u32) * (1 + scale as u32)) >> 8) as u16
}

/// Linear interpolation between `a` and `b` with `frac` in 0..=255.
#[inline]
pub fn lerp8by8(a: u8, b: u8, frac: u8) -> u8 {
    if b >= a {
        a + scale8(b - a, frac)
    } else {
        a - scale8(a - b, frac)
    }
}

/// Saturating u8 add.
#[inline]
pub fn qadd8(a: u8, b: u8) -> u8 {
    a.saturating_add(b)
}

/// Piecewise-linear approximation of cubic ease in/out.
#[inline]
pub fn ease8_in_out_approx(i: u8) -> u8 {
    if i < 64 {
        i / 2
    } else if i > 255 - 64 {
        let j = 255 - i;
        255 - j / 2
    } else {
        let j = i - 64;
        ((j as u16 * 3) / 2) as u8 + 32
    }
}

/// Half-difference used by Ripple/Reactive: signed (a - b), halved, then
/// absolute value.
#[inline]
pub fn abs_half_diff(a: u8, b: u8) -> u8 {
    let d = (a as i16) - (b as i16);
    (d / 2).unsigned_abs() as u8
}
