//! LED geometry description.
//!
//! Effects consume (x, y) positions for each LED and, for [`LedLayout::center`],
//! a centre point used by the Vortex effect. Positions live on the 0..=255
//! grid. Typical keyboards pick an origin in one corner and scale so the
//! opposite corner lands near 255.

/// LED positions for a keyboard. Implementations can be as trivial as a
/// `&'static [(u8, u8)]` slice (see [`SliceLayout`]).
pub trait LedLayout {
    /// Number of LEDs. Must equal the length of the output buffer passed
    /// to [`crate::effects`] functions.
    fn count(&self) -> usize;

    /// `(x, y)` of LED `index` in 0..=255 space.
    fn position(&self, index: usize) -> (u8, u8);

    /// Centre point for effects that operate in polar coordinates (Vortex).
    /// The default averages the min/max of each axis, which matches the
    /// `k_rgb_matrix_center` convention for rectangular layouts.
    fn center(&self) -> (u8, u8) {
        let (mut min_x, mut min_y) = (u8::MAX, u8::MAX);
        let (mut max_x, mut max_y) = (0u8, 0u8);
        for i in 0..self.count() {
            let (x, y) = self.position(i);
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }
        (
            ((min_x as u16 + max_x as u16) / 2) as u8,
            ((min_y as u16 + max_y as u16) / 2) as u8,
        )
    }

    /// Maximum y-value across all LEDs. Gradient uses this to normalise
    /// the vertical ramp.
    fn y_max(&self) -> u8 {
        let mut m = 0u8;
        for i in 0..self.count() {
            let (_, y) = self.position(i);
            if y > m {
                m = y;
            }
        }
        m
    }
}

/// Adapter that treats any `&[(u8, u8)]` as a layout. Index i gives position i.
pub struct SliceLayout<'a> {
    pub positions: &'a [(u8, u8)],
    pub center: Option<(u8, u8)>,
}

impl<'a> SliceLayout<'a> {
    pub const fn new(positions: &'a [(u8, u8)]) -> Self {
        Self {
            positions,
            center: None,
        }
    }

    pub const fn with_center(positions: &'a [(u8, u8)], center: (u8, u8)) -> Self {
        Self {
            positions,
            center: Some(center),
        }
    }
}

impl<'a> LedLayout for SliceLayout<'a> {
    fn count(&self) -> usize {
        self.positions.len()
    }

    fn position(&self, index: usize) -> (u8, u8) {
        self.positions[index]
    }

    fn center(&self) -> (u8, u8) {
        if let Some(c) = self.center {
            c
        } else {
            let (mut min_x, mut min_y) = (u8::MAX, u8::MAX);
            let (mut max_x, mut max_y) = (0u8, 0u8);
            for &(x, y) in self.positions {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
            (
                ((min_x as u16 + max_x as u16) / 2) as u8,
                ((min_y as u16 + max_y as u16) / 2) as u8,
            )
        }
    }
}
