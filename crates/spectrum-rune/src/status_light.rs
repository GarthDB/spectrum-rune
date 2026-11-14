//! Spectrum Status light, from the `status-light` recipe in `platforms/rune/extensions/visuals/status-light.json`
//! (the foundation has none yet; see the manifest README): container > dot, label. The dot is aligned with the top
//! of the label's first line, both starting at the block padding. Every size, colour and type value comes from [`Tokens`].
//!
//! Variants: the five semantic ones and the named colours, as in the Figma Web POC.
//!
//! Verification: geometry, dot fill and label colour against the oracle (`tests/status_light.rs`), and label
//! wrapping (`tests/wrap.rs`). The font family is not asserted (not observable).

use crate::font;
use crate::size::Size;
use crate::tokens::Tokens;
use crate::wrap::Fit;
use rune::{el, row, text, Color, El};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Informative,
    Accent,
    Neutral,
    Positive,
    Notice,
    Negative,
    Indigo,
    Celery,
    Chartreuse,
    Yellow,
    Magenta,
    Fuchsia,
    Purple,
    Seafoam,
    Pink,
    Turquoise,
    Cinnamon,
    Brown,
    Silver,
    Cyan,
    Gray,
    Red,
    Orange,
    Green,
    Blue,
}

impl Variant {
    fn dot(self, t: &Tokens) -> Color {
        match self {
            Variant::Informative => t.informative_visual_color,
            Variant::Accent => t.accent_visual_color,
            Variant::Neutral => t.neutral_visual_color,
            Variant::Positive => t.positive_visual_color,
            Variant::Notice => t.notice_visual_color,
            Variant::Negative => t.negative_visual_color,
            Variant::Indigo => t.indigo_visual_color,
            Variant::Celery => t.celery_visual_color,
            Variant::Chartreuse => t.chartreuse_visual_color,
            Variant::Yellow => t.yellow_visual_color,
            Variant::Magenta => t.magenta_visual_color,
            Variant::Fuchsia => t.fuchsia_visual_color,
            Variant::Purple => t.purple_visual_color,
            Variant::Seafoam => t.seafoam_visual_color,
            Variant::Pink => t.pink_visual_color,
            Variant::Turquoise => t.turquoise_visual_color,
            Variant::Cinnamon => t.cinnamon_visual_color,
            Variant::Brown => t.brown_visual_color,
            Variant::Silver => t.silver_visual_color,
            Variant::Gray => t.gray_visual_color,
            Variant::Red => t.red_visual_color,
            Variant::Orange => t.orange_visual_color,
            Variant::Green => t.green_visual_color,
            Variant::Cyan => t.cyan_visual_color,
            Variant::Blue => t.blue_visual_color,
        }
    }
}

pub struct StatusLight {
    key: String,
    label: String,
    variant: Variant,
    size: Size,
    width: Option<f32>,
}

impl StatusLight {
    /// `key` must be unique per instance: the parts are keyed `{key}/{recipe part}` (`container`, `dot`, `label`).
    /// A status light always has a label: the dot alone would say nothing without its colour.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        StatusLight {
            key: key.into(),
            label: label.into(),
            variant: Variant::default(),
            size: Size::M,
            width: None,
        }
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// Request width in points, like the oracle's `--width`: a label too long for it wraps.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// The root comes back inside a plain `row()` wrapper (see `crate::hug`), so it hugs its content in a column.
    pub fn build(self, t: &Tokens) -> El {
        let (gap, dot) = match self.size {
            Size::S => (t.base_gap_small, t.status_dot_size_small),
            Size::M => (t.base_gap_medium, t.status_dot_size_medium),
            Size::L => (t.base_gap_large, t.status_dot_size_large),
            Size::XL => (t.base_gap_extra_large, t.status_dot_size_extra_large),
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        // shortcut: the recipe binds the ratio `line-height-100` (1.3 x the font size) because status-light has no
        // `line-height-font-size-*` relationships (adobe/spectrum-design-data#1756); Figma uses those per-size values.
        let mut label_type = self.size.label_type(t);
        label_type.line_height = label_type.size * t.line_height_100;
        // No minimum height and no block padding: the component is as tall as its lines, and the dot is
        // centred on the first one.
        let fit = Fit::new(
            Some(&self.label),
            font::text_style(&label_type),
            label_type.line_height,
            dot,
            label_type.line_height,
            self.width.map(|w| w - dot - gap),
        );
        let circle = el()
            .key(part("dot"))
            .size(dot, dot)
            .radius(t.corner_radius_full * dot)
            .fill(self.variant.dot(t));
        let mut label = text(self.label.clone())
            .key(part("label"))
            .color(t.neutral_content_color_default)
            .font(font::text_style(&label_type));
        if self.width.is_some() {
            label = label.wrap().shrink();
        }
        let mut root = fit
            .root(row())
            .key(part("container"))
            .h(fit.height)
            .gap(gap)
            .child(fit.part(circle, dot))
            .child(label);
        if let Some(w) = self.width {
            root = root.w(w);
        }
        crate::hug(root)
    }
}
