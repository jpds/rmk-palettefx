//! RIPPLE: up to 3 radial drops with amplitude envelope.

use super::FrameParams;
use crate::color::Hsv;
use crate::layout::LedLayout;
use crate::math::{abs_half_diff, cos8, ease8_in_out_approx, scale8, scale16by8, sqrt16};
use crate::palette::interp_color;

const RIPPLE_DROPS: usize = 3;
const RIPPLE_SPAWN_INTERVAL_MS: u32 = 1000;

#[derive(Copy, Clone, Default)]
struct Droplet {
    /// Timer value (millisecond counter, low 32 bits) when the drop spawned.
    spawn_ms: u32,
    x: u8,
    y: u8,
    amplitude: u8,
    scale: u8,
    phase: u8,
}

pub struct RippleState {
    drops: [Droplet; RIPPLE_DROPS],
    drops_tail: usize,
    next_spawn_ms: u32,
    initialized: bool,
}

impl Default for RippleState {
    fn default() -> Self {
        Self::new()
    }
}

impl RippleState {
    pub const fn new() -> Self {
        Self {
            drops: [Droplet {
                spawn_ms: 0,
                x: 0,
                y: 0,
                amplitude: 0,
                scale: 0,
                phase: 0,
            }; RIPPLE_DROPS],
            drops_tail: 0,
            next_spawn_ms: 0,
            initialized: false,
        }
    }

    /// Tick the ripple effect. `rng()` is called whenever a new drop is
    /// spawned and should return a byte uniformly over 0..=255. The LED
    /// index `rng() % led_count` becomes the drop centre.
    pub fn tick<L, R>(&mut self, layout: &L, params: FrameParams<'_>, mut rng: R, out: &mut [Hsv])
    where
        L: LedLayout,
        R: FnMut() -> u8,
    {
        let count = layout.count();

        if !self.initialized {
            self.initialized = true;
            self.next_spawn_ms = params.timer_ms;
        }

        // Spawn a new drop if the slot at `drops_tail` is free and the
        // inter-drop timer has elapsed.
        if self.drops[self.drops_tail].amplitude == 0
            && params.timer_ms.wrapping_sub(self.next_spawn_ms) < u32::MAX / 2
        {
            let led = (rng() as usize) % count.max(1);
            let (dx, dy) = layout.position(led);
            let slot = self.drops_tail;
            self.drops[slot] = Droplet {
                spawn_ms: params.timer_ms,
                x: dx,
                y: dy,
                amplitude: 1,
                scale: 0,
                phase: 0,
            };
            self.drops_tail = (slot + 1) % RIPPLE_DROPS;
            self.next_spawn_ms = params.timer_ms.wrapping_add(RIPPLE_SPAWN_INTERVAL_MS);
        }

        // Advance each active droplet.
        for droplet in &mut self.drops {
            if droplet.amplitude == 0 {
                continue;
            }
            let elapsed = params.timer_ms.wrapping_sub(droplet.spawn_ms) as u16;
            let tick = scale16by8(elapsed, 1 + params.speed / 4);
            if tick < 4 * 255 {
                let t = (tick / 4) as u8;
                droplet.amplitude = ripple_amplitude(t);
                droplet.scale = 255 / (1 + t / 2);
                droplet.phase = tick as u8;
            } else {
                droplet.amplitude = 0;
            }
        }

        // Render.
        for (i, slot) in out.iter_mut().take(count).enumerate() {
            let (lx, ly) = layout.position(i);
            let mut value: i16 = 128;

            for droplet in &self.drops {
                if droplet.amplitude == 0 {
                    continue;
                }
                let dx = abs_half_diff(lx, droplet.x);
                let dy = abs_half_diff(ly, droplet.y);
                let r = sqrt16((dx as u16) * (dx as u16) + (dy as u16) * (dy as u16));
                let r_scaled = (r as u16) * (droplet.scale as u16);

                if r_scaled < 255 {
                    let bump = scale8(ease8_in_out_approx(255 - r_scaled as u8), droplet.amplitude);
                    let wave = (cos8(8u8.wrapping_mul(r).wrapping_sub(droplet.phase)) as i16) - 128;
                    value += (wave * (bump as i16)) / 128;
                }
            }

            let value = value.clamp(0, 255) as u8;
            *slot = interp_color(params.palette, value, params.sat, params.val);
        }
    }
}

/// Droplet amplitude envelope: rising for t<32, plateau to t=55, then a smooth
/// quadratic-ish decay back to zero at t=255.
fn ripple_amplitude(t: u8) -> u8 {
    if t <= 55 {
        if t < 32 { 3 + 5 * t } else { 192 }
    } else {
        let u = (((255 - t) as u16) * 123) >> 7;
        scale8(u as u8, u as u8)
    }
}
