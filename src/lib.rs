//! # rmk-palettefx
//!
//! A Rust, `no_std` port of Pascal Getreuer's
//! [PaletteFx](https://getreuer.info/posts/keyboards/palettefx) module.
//! Replaces single-hue RGB effects with palette-sampled ones:
//! each effect produces a scalar 0..=255 per LED, then samples a 16-stop
//! gradient palette via linear interpolation.
//!
//! The crate is decoupled from any LED driver. Effect functions write
//! [`Hsv`](color::Hsv) triples into a caller-supplied output slice; the
//! caller converts to RGB (via [`color::hsv_to_rgb`] or equivalent) and
//! pushes the result to hardware.
//!
//! ## Layering
//!
//! - [`palette`]: 16 built-in gradient palettes and the `interp_color`
//!   lookup. Palettes are flat `[u16; 16]` arrays so they live in `.rodata`.
//! - [`color`]: 8-bit [`Hsv`](color::Hsv)/[`Rgb`](color::Rgb) types and a
//!   spectrum HSV→RGB converter.
//! - [`math`]: `sin8`/`cos8` LUTs, `atan2_8`, `sqrt16`, and 8-bit
//!   fixed-point helpers used by the effects.
//! - [`time`]: [`TimePhase`](time::TimePhase), the wraparound-safe scaled
//!   time accumulator.
//! - [`layout`]: [`LedLayout`](layout::LedLayout) trait describing the
//!   (x, y) position of each LED in 0..=255 space.
//! - [`effects`]: the six PaletteFx effects (Gradient, Flow, Ripple,
//!   Sparkle, Vortex, Reactive). Stateful effects (Flow, Ripple, Sparkle,
//!   Vortex, Reactive) carry their state in a struct; Gradient is a free
//!   function.
//!
//! ## Minimal example
//!
//! ```ignore
//! use rmk_palettefx::color::{Hsv, hsv_to_rgb};
//! use rmk_palettefx::effects::{FrameParams, gradient};
//! use rmk_palettefx::layout::SliceLayout;
//! use rmk_palettefx::palette::CARNIVAL;
//!
//! const POSITIONS: &[(u8, u8)] = &[(0, 0), (64, 0), (128, 0)];
//! let layout = SliceLayout::new(POSITIONS);
//! let mut frame = [Hsv::default(); 3];
//! gradient(
//!     &layout,
//!     FrameParams {
//!         palette: &CARNIVAL,
//!         speed: 0,
//!         sat: 255,
//!         val: 255,
//!         timer_ms: 0,
//!     },
//!     &mut frame,
//! );
//! for hsv in frame {
//!     let _rgb = hsv_to_rgb(hsv);
//!     // ... push to driver
//! }
//! ```

#![no_std]
#![forbid(unsafe_code)]

pub mod color;
pub mod effect_state;
pub mod effects;
pub mod layout;
pub mod led_driver;
pub mod math;
pub mod palette;
pub mod time;

pub use effect_state::EffectState;
pub use led_driver::LedDriver;
