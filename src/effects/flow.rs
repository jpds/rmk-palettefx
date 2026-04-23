//! FLOW: rotating space with interfering sines.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::math::{cos8, scale8, sin8};
use crate::palette::interp_color;
use crate::time::TimePhase;

#[derive(Default)]
pub struct FlowState {
    time_phase: TimePhase,
}

impl FlowState {
    pub const fn new() -> Self {
        Self {
            time_phase: TimePhase::new(),
        }
    }

    pub fn tick<L: LedLayout>(&mut self, layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
        let time = self
            .time_phase
            .update(params.timer_ms, 1 + params.speed / 8);
        // `time/4` and `time` (low byte) feed the 8-bit trig LUTs.
        let rot_c = (cos8((time >> 2) as u8) as i16) - 128;
        let rot_s = (sin8((time >> 2) as u8) as i16) - 128;
        let omega = 32u8.wrapping_add(sin8(time as u8) / 4);

        for (i, slot) in out.iter_mut().enumerate() {
            let (x, y) = layout.position(i);
            let xi = x as i16;
            let yi = y as i16;

            // Rotate (x, y) by the 2×2 matrix [c -s; s c] with 7 fractional bits.
            let x1 = ((rot_c * xi) / 128) as u8;
            let x1 = x1.wrapping_sub(((rot_s * yi) / 128) as u8);
            let y1 = ((rot_s * xi) / 128) as u8;
            let y1 = y1.wrapping_add(((rot_c * yi) / 128) as u8);

            let phase_in = x1.wrapping_sub((time as u8).wrapping_mul(2));
            let mut value = scale8(sin8(phase_in), omega)
                .wrapping_add(y1)
                .wrapping_add((time >> 2) as u8);
            // Sawtooth fold: tent function on [0, 255].
            value = if value <= 127 {
                value.wrapping_mul(2)
            } else {
                (255 - value).wrapping_mul(2)
            };

            *slot = interp_color(params.palette, value, params.sat, params.val);
        }
    }
}
