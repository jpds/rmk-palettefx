//! GRADIENT: vertical ramp, time-independent.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::palette::interp_color;

/// Static vertical gradient: top-most LEDs sample the palette near 255, the
/// bottom row near 0.
pub fn gradient<L: LedLayout>(layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
    let y_max = layout.y_max().max(64);
    // 255 / y_max with 6 fractional bits and rounding.
    let slope = ((64u32 * 255 + (y_max as u32) / 2) / y_max as u32) as u16;

    for (i, slot) in out.iter_mut().enumerate() {
        let (_, y) = layout.position(i);
        let value = 255 - (((y as u32) * (slope as u32)) >> 6) as u8;
        *slot = interp_color(params.palette, value, params.sat, params.val);
    }
}
