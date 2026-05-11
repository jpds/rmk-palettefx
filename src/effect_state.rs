//! Runtime-switchable effect dispatch.
//!
//! [`EffectState`] wraps every effect's state struct in a single enum so the
//! active effect can be changed without reallocating. `tick` dispatches to the
//! variant's renderer; `record_hit` is meaningful only for [`EffectState::Reactive`]
//! and is a no-op for every other variant.

use rand_pcg::Pcg32;

use crate::color::Hsv;
use crate::effects::{
    FlowState, FrameParams, ReactiveState, RippleState, SparkleState, VortexState, gradient,
};
use crate::layout::LedLayout;

/// Default seed for the Ripple effect's internal RNG. Picked arbitrarily; any
/// non-zero value works.
const DEFAULT_RIPPLE_SEED: u64 = 0xA5A5_5A5A_DEAD_BEEF;

/// All built-in effects, switchable at runtime.
///
/// `N_HITS` is the per-frame hit history depth used by
/// [`EffectState::Reactive`]. Other variants ignore it.
pub enum EffectState<const N_HITS: usize> {
    Gradient,
    Flow(FlowState),
    Ripple(RippleState, Pcg32),
    Sparkle(SparkleState),
    Vortex(VortexState),
    Reactive(ReactiveState<N_HITS>),
}

impl<const N_HITS: usize> EffectState<N_HITS> {
    pub fn flow() -> Self {
        Self::Flow(FlowState::new())
    }

    pub fn ripple_seeded(seed: u64) -> Self {
        Self::Ripple(RippleState::new(), Pcg32::new(seed, 0x0A02_BDBF_7BB3_C0A7))
    }

    pub fn ripple() -> Self {
        Self::ripple_seeded(DEFAULT_RIPPLE_SEED)
    }

    pub fn sparkle() -> Self {
        Self::Sparkle(SparkleState::new())
    }

    pub fn vortex() -> Self {
        Self::Vortex(VortexState::new())
    }

    pub fn reactive() -> Self {
        Self::Reactive(ReactiveState::new())
    }

    /// Render one frame.
    pub fn tick<L: LedLayout>(&mut self, layout: &L, params: FrameParams<'_>, out: &mut [Hsv]) {
        match self {
            Self::Gradient => gradient(layout, params, out),
            Self::Flow(s) => s.tick(layout, params, out),
            Self::Ripple(s, rng) => s.tick_with_rng(rng, layout, params, out),
            Self::Sparkle(s) => s.tick(layout, params, out),
            Self::Vortex(s) => s.tick(layout, params, out),
            Self::Reactive(s) => s.tick(layout, params, out),
        }
    }

    /// Record a key press at `(x, y)` in layout coordinates. No-op for
    /// non-reactive variants.
    pub fn record_hit(&mut self, x: u8, y: u8, timer_ms: u32) {
        if let Self::Reactive(s) = self {
            s.record_hit(x, y, timer_ms);
        }
    }
}
