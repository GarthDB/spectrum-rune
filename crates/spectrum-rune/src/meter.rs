//! Spectrum Meter: a bar showing a quantity within a range (disk used, a score), with a label, a value, a status colour
//! and optional help text, from `design-data` visuals/meter.json (the progress bar's layout with a variant colour). Every
//! size, colour and type value comes from [`Tokens`].
//!
//! Built: `label` and `hide_label`, `value` with `min_value` / `max_value` (the value text is the percentage, or
//! [`Meter::value_label`]), `width`, `help_text`, the four sizes and the four variants. Not built: `staticColor`.
//!
//! Verification: geometry, type and colours against the oracle (`tests/meter.rs`).

use crate::bar::Bar;
use crate::tokens::Tokens;
pub use crate::Size;
use rune::El;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Informative,
    Positive,
    Notice,
    Negative,
}

pub struct Meter {
    key: String,
    size: Size,
    variant: Variant,
    label: Option<String>,
    hide_label: bool,
    value: f32,
    min: f32,
    max: f32,
    value_label: Option<String>,
    help: Option<String>,
    width: Option<f32>,
}

impl Meter {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `header`, `label`, `value`, `track`, `fill`
    /// and `help-text`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        Meter {
            key: key.into(),
            size: Size::M,
            variant: Variant::default(),
            label: None,
            hide_label: false,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            value_label: None,
            help: None,
            width: None,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    /// Shown above the bar and read as its accessible name. Every meter should have one.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Hides the label and the value (the accessible name stays).
    pub fn hide_label(mut self, v: bool) -> Self {
        self.hide_label = v;
        self
    }
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }
    pub fn min_value(mut self, v: f32) -> Self {
        self.min = v;
        self
    }
    pub fn max_value(mut self, v: f32) -> Self {
        self.max = v;
        self
    }
    /// Replaces the percentage shown as the value.
    pub fn value_label(mut self, l: impl Into<String>) -> Self {
        self.value_label = Some(l.into());
        self
    }
    pub fn help_text(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let span = (self.max - self.min).max(f32::EPSILON);
        let fraction = ((self.value - self.min) / span).clamp(0.0, 1.0);
        let value_text = if self.hide_label {
            None
        } else {
            Some(
                self.value_label
                    .clone()
                    .unwrap_or_else(|| format!("{}%", (fraction * 100.0).round())),
            )
        };
        let fill = match self.variant {
            Variant::Informative => t.informative_visual_color,
            Variant::Positive => t.positive_visual_color,
            Variant::Notice => t.notice_visual_color,
            Variant::Negative => t.negative_visual_color,
        };
        let name = self.label.clone().unwrap_or_else(|| "Meter".to_owned());
        let a11y = value_text.clone().unwrap_or_default();
        Bar {
            key: self.key,
            size: self.size,
            label: if self.hide_label { None } else { self.label },
            value_text,
            help: self.help,
            width: self.width,
            fraction: Some(fraction),
            fill,
            role_label: name,
        }
        .build(t)
        .range_value(f64::from(fraction * 100.0), 0.0, 100.0)
        .a11y_value(a11y)
    }
}
