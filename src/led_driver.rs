//! Hardware-facing LED driver trait.
//!
//! [`LedDriver`] is the boundary between rmk-palettefx (which only computes
//! [`Hsv`](crate::color::Hsv) buffers) and the user's LED hardware (WS2812,
//! APA102, PWM channels, etc). The crate's processor calls [`LedDriver::write`]
//! once per rendered frame; the implementation is responsible for converting
//! to whatever wire format the chip wants and pushing the bytes out.

use crate::color::Hsv;

/// Push a fully rendered HSV frame to LED hardware.
///
/// Implementations typically convert each [`Hsv`] to RGB (see
/// [`color::hsv_to_rgb`](crate::color::hsv_to_rgb)) and then emit it over
/// whatever transport the chip uses. `write` is awaited once per frame so it
/// may freely use async SPI/PIO/DMA APIs.
#[allow(async_fn_in_trait)]
pub trait LedDriver {
    async fn write(&mut self, leds: &[Hsv]);
}
