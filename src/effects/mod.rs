//! The six PaletteFx effects. Each effect writes one [`Hsv`](crate::color::Hsv)
//! per LED into the caller-supplied output slice; converting to RGB and
//! pushing to the LED driver is the caller's responsibility.

use crate::palette::Palette;

mod flow;
mod gradient;
mod reactive;
mod ripple;
mod sparkle;
mod vortex;

pub use flow::FlowState;
pub use gradient::gradient;
pub use reactive::{Hit, ReactiveState};
pub use ripple::RippleState;
pub use sparkle::SparkleState;
pub use vortex::VortexState;

/// Parameters shared by every effect.
#[derive(Copy, Clone, Debug)]
pub struct FrameParams<'p> {
    pub palette: &'p Palette,
    /// 0..=255, maps to PaletteFx's `rgb_matrix_config.speed`.
    pub speed: u8,
    /// Global saturation scale. Pass 255 to leave the palette unmodified.
    pub sat: u8,
    /// Global brightness/value scale. Pass 255 for full brightness.
    pub val: u8,
    /// Monotonic millisecond counter.
    pub timer_ms: u32,
}
