# rmk-palettefx

A `no_std` Rust port of Pascal Getreuer's QMK
[PaletteFx](https://getreuer.info/posts/keyboards/palettefx) community module
for [RMK](https://rmk.rs/) keyboard firmware.

Effects write HSV values into a caller-supplied frame buffer. The
caller converts to RGB and ships the result to whatever LED driver
they have wired up. This crate is MCU, HAL, and driver agnostic.

## Effects

Six effects, all using the 16 built-in palettes:

- Flow
- Gradient
- Reactive
- Ripple
- Sparkle
- Vortex

## Reference consumer

[rmk-zsa-voyager](https://github.com/jpds/rmk-zsa-voyager) drives the
ZSA Voyager's 52-key per-key RGB through this crate. See its codebase for an
end-to-end consumer.

## Credits

All palette data and effect algorithms come from Pascal Getreuer's
[QMK PaletteFx module](https://github.com/getreuer/qmk-modules/tree/main/palettefx).

## License

[Apache-2.0](LICENSE).
