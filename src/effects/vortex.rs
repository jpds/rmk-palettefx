//! VORTEX: polar spiral centred on the layout centre.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::math::{atan2_8, sin8, sqrt16};
use crate::palette::interp_color;
use crate::time::TimePhase;

#[derive(Default)]
pub struct VortexState {
    time_phase: TimePhase,
}

impl VortexState {
    pub const fn new() -> Self {
        Self {
            time_phase: TimePhase::new(),
        }
    }

    pub fn tick<L: LedLayout>(&mut self, layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
        let time = self
            .time_phase
            .update(params.timer_ms, 1 + params.speed / 4);
        let (cx, cy) = layout.center();

        for (i, slot) in out.iter_mut().enumerate() {
            let (lx, ly) = layout.position(i);
            let x = lx as i16 - cx as i16;
            let y = ly as i16 - cy as i16;
            let r = sqrt16((x * x + y * y) as u16);
            let phase = atan2_8(y, x).wrapping_add(time as u8).wrapping_sub(r / 2);
            let value = sin8(phase);
            *slot = interp_color(params.palette, value, params.sat, params.val);
        }
    }
}
