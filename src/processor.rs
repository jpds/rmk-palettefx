//! [`PaletteFxProcessor`]: bridge between rmk's [`PollingProcessor`] machinery
//! and rmk-palettefx's effect renderers.
//!
//! Available only when the `rmk` cargo feature is enabled.
//!
//! ## Wiring
//!
//! The processor:
//! - subscribes to [`KeyboardEvent`] so it can feed reactive effects with hit
//!   positions, and
//! - is polled at a fixed interval (default ~60 Hz) by rmk's runtime, at which
//!   point it renders the active effect into a fixed-size buffer and forwards
//!   the buffer to a user-supplied [`LedDriver`].
//!
//! ```rust,ignore
//! use embassy_futures::join::join;
//! use rmk_palettefx::{LedDriver, PaletteFxProcessor, EffectState, palette::CARNIVAL};
//! use rmk_palettefx::layout::SliceLayout;
//!
//! static POSITIONS: &[(u8, u8)] = &[/* ... */];
//! const LEDS: usize = /* ... */;
//!
//! fn key_to_pos(row: u8, col: u8) -> Option<(u8, u8)> { /* board-specific */ }
//!
//! let layout = SliceLayout::new(POSITIONS);
//! let mut fx = PaletteFxProcessor::<_, LEDS, 8>::new(
//!     my_driver, layout, EffectState::flow(), &CARNIVAL, key_to_pos,
//! );
//!
//! join(fx.polling_loop(), run_rmk(/* ... */)).await;
//! ```

use embassy_time::Instant;
use rmk::event::{KeyboardEvent, KeyboardEventPos};
use rmk_macro::processor;

use crate::color::Hsv;
use crate::effect_state::EffectState;
use crate::effects::FrameParams;
use crate::layout::LedLayout;
use crate::led_driver::LedDriver;
use crate::palette::Palette;

/// Maps a `(row, col)` matrix position to an `(x, y)` LED-layout coordinate.
///
/// Returning `None` means "this matrix position has no associated LED" and the
/// event is silently ignored. Each board provides its own implementation
/// alongside its [`LedLayout`].
pub type KeyToPos = fn(row: u8, col: u8) -> Option<(u8, u8)>;

/// rmk processor that runs a [`PaletteFx`](crate) effect on `LEDS` LEDs with
/// up to `HITS` outstanding reactive hits.
///
/// - `D`: hardware-specific [`LedDriver`].
/// - `L`: layout describing each LED's `(x, y)` position.
/// - `LEDS`: LED count. Must match `L::count()`.
/// - `HITS`: capacity of the reactive-effect hit ring (ignored for other
///   effects; pick `0` if reactive is never used).
#[processor(subscribe = [KeyboardEvent], poll_interval = 16)]
pub struct PaletteFxProcessor<D, L, const LEDS: usize, const HITS: usize>
where
    D: LedDriver,
    L: LedLayout,
{
    driver: D,
    layout: L,
    effect: EffectState<HITS>,
    palette: &'static Palette,
    speed: u8,
    sat: u8,
    val: u8,
    buffer: [Hsv; LEDS],
    key_to_pos: KeyToPos,
}

impl<D, L, const LEDS: usize, const HITS: usize> PaletteFxProcessor<D, L, LEDS, HITS>
where
    D: LedDriver,
    L: LedLayout,
{
    /// Construct with full brightness/saturation and a mid speed. Tune via
    /// [`Self::set_speed`] / [`Self::set_sat`] / [`Self::set_val`].
    pub fn new(
        driver: D,
        layout: L,
        effect: EffectState<HITS>,
        palette: &'static Palette,
        key_to_pos: KeyToPos,
    ) -> Self {
        Self {
            driver,
            layout,
            effect,
            palette,
            speed: 128,
            sat: 255,
            val: 255,
            buffer: [Hsv { h: 0, s: 0, v: 0 }; LEDS],
            key_to_pos,
        }
    }

    pub fn set_effect(&mut self, effect: EffectState<HITS>) {
        self.effect = effect;
    }

    pub fn set_palette(&mut self, palette: &'static Palette) {
        self.palette = palette;
    }

    pub fn set_speed(&mut self, speed: u8) {
        self.speed = speed;
    }

    pub fn set_sat(&mut self, sat: u8) {
        self.sat = sat;
    }

    pub fn set_val(&mut self, val: u8) {
        self.val = val;
    }

    async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        if !event.pressed {
            return;
        }
        let KeyboardEventPos::Key(pos) = event.pos else {
            return;
        };
        if let Some((x, y)) = (self.key_to_pos)(pos.row, pos.col) {
            let ms = Instant::now().as_millis() as u32;
            self.effect.record_hit(x, y, ms);
        }
    }

    async fn poll(&mut self) {
        let ms = Instant::now().as_millis() as u32;
        let params = FrameParams {
            palette: self.palette,
            speed: self.speed,
            sat: self.sat,
            val: self.val,
            timer_ms: ms,
        };
        self.effect.tick(&self.layout, params, &mut self.buffer);
        self.driver.write(&self.buffer).await;
    }
}
