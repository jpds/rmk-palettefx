//! Wraparound-safe scaled time, ported from `palettefx_scaled_time`.
//!
//! Naively evaluating `(timer_ms * scale) >> 8` as a 16-bit quantity produces
//! a visible step each time the u32 millisecond counter rolls a byte boundary.
//! [`TimePhase`] carries a per-instance correction term so the scaled phase
//! advances smoothly across the boundary.

use crate::math::scale16by8;

/// Per-effect time scaler. Each animated effect gets its own instance so
/// independent speed values don't contaminate each other's correction.
#[derive(Copy, Clone, Debug, Default)]
pub struct TimePhase {
    wrap_correction: u16,
    last_high_byte: u8,
}

impl TimePhase {
    pub const fn new() -> Self {
        Self {
            wrap_correction: 0,
            last_high_byte: 0,
        }
    }

    /// Compute a 16-bit animation phase for `timer_ms` scaled by `scale`.
    /// Pass a monotonic millisecond counter; this method mutates internal
    /// state to keep phase smooth across the u32 rollover.
    pub fn update(&mut self, timer_ms: u32, scale: u8) -> u16 {
        let high_byte = (timer_ms >> 16) as u8;
        if self.last_high_byte != high_byte {
            self.last_high_byte = high_byte;
            self.wrap_correction = self.wrap_correction.wrapping_add((scale as u16) << 8);
        }
        scale16by8(timer_ms as u16, scale).wrapping_add(self.wrap_correction)
    }
}
