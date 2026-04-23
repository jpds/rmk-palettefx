//! REACTIVE: radial bumps around recent key hits.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::math::{abs_half_diff, qadd8, scale8, scale16by8, sqrt16};
use crate::palette::interp_color;

/// A single key-hit record. `x`/`y` are the LED coordinates of the pressed
/// key; `spawn_ms` is when the hit occurred.
#[derive(Copy, Clone, Default)]
pub struct Hit {
    pub x: u8,
    pub y: u8,
    pub spawn_ms: u32,
    active: bool,
}

/// Reactive state with `N` hit slots.
pub struct ReactiveState<const N: usize> {
    hits: [Hit; N],
    /// Next slot to overwrite (oldest hit).
    next: usize,
}

impl<const N: usize> Default for ReactiveState<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ReactiveState<N> {
    pub const fn new() -> Self {
        Self {
            hits: [Hit {
                x: 0,
                y: 0,
                spawn_ms: 0,
                active: false,
            }; N],
            next: 0,
        }
    }

    /// Record a key press at `(x, y)`. The oldest slot is overwritten if all
    /// are in use.
    pub fn record_hit(&mut self, x: u8, y: u8, timer_ms: u32) {
        if N == 0 {
            return;
        }
        self.hits[self.next] = Hit {
            x,
            y,
            spawn_ms: timer_ms,
            active: true,
        };
        self.next = (self.next + 1) % N;
    }

    pub fn tick<L: LedLayout>(&mut self, layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
        // Precompute each active hit's current amplitude.
        let mut hit_amplitude = [0u8; N];
        for (j, amp) in hit_amplitude.iter_mut().enumerate() {
            if !self.hits[j].active {
                continue;
            }
            let elapsed = params.timer_ms.wrapping_sub(self.hits[j].spawn_ms) as u16;
            let tick = scale16by8(elapsed, 1 + params.speed / 4);
            if tick <= 255 {
                *amp = reactive_amplitude(tick as u8);
            } else {
                self.hits[j].active = false;
            }
        }

        for (i, slot) in out.iter_mut().enumerate() {
            let (lx, ly) = layout.position(i);
            let mut value: u8 = 0;

            for (j, &amp) in hit_amplitude.iter().enumerate() {
                if amp == 0 {
                    continue;
                }
                let dx = abs_half_diff(lx, self.hits[j].x);
                let dy = abs_half_diff(ly, self.hits[j].y);
                if dx < 21 && dy < 21 {
                    let dist_sqr = (dx as u16) * (dx as u16) + (dy as u16) * (dy as u16);
                    if dist_sqr < 21 * 21 {
                        let dist = sqrt16(dist_sqr);
                        value = qadd8(
                            value,
                            scale8(255u8.wrapping_sub(12u8.wrapping_mul(dist)), amp),
                        );
                        if value == 255 {
                            break;
                        }
                    }
                }
            }

            let mut hsv = interp_color(params.palette, value, params.sat, params.val);
            if value < 32 {
                hsv.v = scale8(hsv.v, 64 + 6 * value);
            }
            *slot = hsv;
        }
    }
}

fn reactive_amplitude(t: u8) -> u8 {
    if t <= 55 {
        if t < 32 { 4 + 8 * t } else { 255 }
    } else {
        let u = (((255 - t) as u16) * 164) >> 7;
        scale8(u as u8, u as u8)
    }
}
