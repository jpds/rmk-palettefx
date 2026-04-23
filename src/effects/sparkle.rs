//! SPARKLE: per-LED sine with pseudo-random phase + global brightening.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::math::{scale8, sin8};
use crate::palette::interp_color;
use crate::time::TimePhase;

#[derive(Default)]
pub struct SparkleState {
    time_phase: TimePhase,
}

impl SparkleState {
    pub const fn new() -> Self {
        Self {
            time_phase: TimePhase::new(),
        }
    }

    pub fn tick<L: LedLayout>(&mut self, layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
        let time = self
            .time_phase
            .update(params.timer_ms, 1 + params.speed / 8) as u8;
        let amplitude = 128u8.wrapping_add(sin8(time) / 2);

        // Each LED has a fixed phase AND a fixed frequency multiplier,
        // both derived from an iterative LCG seeded at 1 (the original
        // palettefx setup for a single-chunk pass). Visible motion
        // comes from per-LED sine sweeps over `time` plus the global
        // amplitude envelope.
        //
        // The original C uses a fixed `2 * time` multiplier for every
        // LED, which at ~120 Hz rendering blends into scattered
        // sparkles. At lower tick rates the shared frequency makes
        // the whole matrix pulse in lockstep: the pattern repeats
        // exactly every `256 / (2 * time_rate)` seconds, which reads
        // as a coordinated wave. Varying the multiplier per LED across
        // {2, 3} keeps the anti-loop coprime mix while staying close
        // to the original uniform 2x mean, so per-LED variance matches
        // the original effect's feel. Wider ranges (e.g. 2..=5) make
        // fast LEDs swing too hard between frames at sub-60 Hz tick rates.
        let mut rand_state: u16 = 1;
        for slot in out.iter_mut().take(layout.count()) {
            rand_state = rand_state.wrapping_mul(36563);
            let phase = (rand_state >> 8) as u8;
            // Take the frequency bit from `phase` rather than the low
            // bits of `rand_state`: for an odd LCG multiplier, the low
            // bits are periodic (they only ever take two distinct
            // values) which collapses the split unevenly. Bit 8 of
            // rand_state (= low bit of phase) is well-mixed.
            let freq = 2u16 + (phase as u16 & 0x01);
            let angle = (time as u16).wrapping_mul(freq) as u8;
            let value = scale8(sin8(angle.wrapping_add(phase)), amplitude);
            *slot = interp_color(params.palette, value, params.sat, params.val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::SliceLayout;
    use crate::palette::CARNIVAL;

    fn params(timer_ms: u32, speed: u8) -> FrameParams<'static> {
        FrameParams {
            palette: &CARNIVAL,
            speed,
            sat: 255,
            val: 255,
            timer_ms,
        }
    }

    /// Repeated `tick` calls with the same `timer_ms` must produce the
    /// same frame. Without this, the per-LED flicker rate depends on
    /// the caller's tick frequency rather than on elapsed time, and
    /// Sparkle looks very different at 20 Hz vs 120 Hz tick rates.
    #[test]
    fn flicker_rate_is_time_derived_not_call_derived() {
        const POS: &[(u8, u8)] = &[(0, 0), (64, 0), (128, 0), (192, 0), (255, 0)];
        let layout = SliceLayout::new(POS);
        let mut state = SparkleState::new();
        let mut out = [Hsv::default(); 5];

        state.tick(&layout, params(100, 128), &mut out);
        let baseline = out;

        state.tick(&layout, params(100, 128), &mut out);
        assert_eq!(out, baseline);
        state.tick(&layout, params(100, 128), &mut out);
        assert_eq!(out, baseline);

        state.tick(&layout, params(10_000, 128), &mut out);
        assert_ne!(out, baseline);
    }
}
