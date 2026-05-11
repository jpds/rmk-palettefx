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

    pub fn ripple(seed: u64) -> Self {
        Self::Ripple(RippleState::new(), Pcg32::new(seed, 0x0A02_BDBF_7BB3_C0A7))
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

    /// Advance to the next effect in declaration order. `ripple_seed` is
    /// only used when the new variant is [`Self::Ripple`]; for other
    /// transitions it is ignored. Callers typically pass a millisecond
    /// timer so successive Ripple cycles drop their first droplet in a
    /// different place.
    pub fn next(&mut self, ripple_seed: u64) {
        *self = match self {
            Self::Gradient => Self::flow(),
            Self::Flow(_) => Self::ripple(ripple_seed),
            Self::Ripple(_, _) => Self::sparkle(),
            Self::Sparkle(_) => Self::vortex(),
            Self::Vortex(_) => Self::reactive(),
            Self::Reactive(_) => Self::Gradient,
        };
    }

    /// Step backward through the same order as [`Self::next`].
    pub fn prev(&mut self, ripple_seed: u64) {
        *self = match self {
            Self::Gradient => Self::reactive(),
            Self::Reactive(_) => Self::vortex(),
            Self::Vortex(_) => Self::sparkle(),
            Self::Sparkle(_) => Self::ripple(ripple_seed),
            Self::Ripple(_, _) => Self::flow(),
            Self::Flow(_) => Self::Gradient,
        };
    }
}
