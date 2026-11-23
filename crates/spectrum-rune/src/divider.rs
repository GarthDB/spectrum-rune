//! Spectrum Divider: a thin line that separates content, from `design-data` visuals/divider.json (one `line` part).
//! Every thickness, length and colour comes from [`Tokens`].
//!
//! Built: the three sizes (`S`, `M`, `L`; thickness 1, 2 and 4 pt on desktop, mobile's own tokens on the touch scale),
//! horizontal and vertical. Without [`Divider::length`] the line fills its parent's width (height, vertically), at least
//! the recipe's minimum of 16 pt. Not built: `staticColor`.
//!
//! Verification: size, radius and colour against the oracle (`tests/divider.rs`).

use crate::tokens::Tokens;
use rune::{el, El};

pub use crate::Size;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

pub struct Divider {
    key: String,
    size: Size,
    orientation: Orientation,
    length: Option<f32>,
}

impl Divider {
    /// `key` is the line's key (the recipe's `line` part): `rt.find("{key}")` works.
    pub fn new(key: impl Into<String>) -> Self {
        Divider {
            key: key.into(),
            size: Size::S,
            orientation: Orientation::default(),
            length: None,
        }
    }
    /// `S`, `M` or `L`; the recipe has no `XL` (it draws as `L`).
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }
    /// A fixed length, instead of filling the parent.
    pub fn length(mut self, l: f32) -> Self {
        self.length = Some(l);
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (thickness, color) = match self.size {
            Size::S => (t.bar_thickness_2x_small, t.gray_200),
            Size::M => (t.bar_thickness_extra_small, t.gray_200),
            Size::L | Size::XL => (t.bar_thickness_small, t.gray_800),
        };
        let min = t.bar_minimum_width_2x_small;
        let line = el().key(self.key).radius(thickness / 2.0).fill(color);
        match (self.orientation, self.length) {
            (Orientation::Horizontal, Some(l)) => line.w(l.max(min)).h(thickness),
            (Orientation::Horizontal, None) => line.w_full().min_w(min).h(thickness),
            (Orientation::Vertical, Some(l)) => line.h(l.max(min)).w(thickness),
            (Orientation::Vertical, None) => line.h_full().min_h(min).w(thickness),
        }
    }
}
