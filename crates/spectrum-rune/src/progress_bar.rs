//! Spectrum Progress bar: a track with a fill showing how far along something is, and a label and value above it, from
//! `design-data` visuals/progress-bar.json. Every size, colour and type value comes from [`Tokens`].
//!
//! Built: `label` and `hide_label`, `value` with `min_value` / `max_value` (the value text is the percentage, or
//! [`ProgressBar::value_label`]), `width` (otherwise the bar is as wide as its header), the four sizes and `indeterminate`,
//! drawn at rest where the oracle's frame has it (the animation is not built). Not built: `staticColor` and
//! `labelPosition: side` (the oracle draws it like `top`).
//!
//! Verification: geometry, type and colours against the oracle (`tests/progress_bar.rs`).

use crate::bar::Bar;
use crate::tokens::Tokens;
pub use crate::Size;
use rune::El;

pub struct ProgressBar {
    key: String,
    size: Size,
    label: Option<String>,
    hide_label: bool,
    value: Option<f32>,
    min: f32,
    max: f32,
    value_label: Option<String>,
    width: Option<f32>,
    indeterminate: bool,
}

impl ProgressBar {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `header`, `label`, `value`, `track` and
    /// `fill`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        ProgressBar {
            key: key.into(),
            size: Size::M,
            label: None,
            hide_label: false,
            value: None,
            min: 0.0,
            max: 100.0,
            value_label: None,
            width: None,
            indeterminate: false,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// Shown above the bar and read as its accessible name. Every bar should have one.
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
        self.value = Some(v);
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
    /// Replaces the percentage shown as the value (`2 of 5`).
    pub fn value_label(mut self, l: impl Into<String>) -> Self {
        self.value_label = Some(l.into());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// The amount of work is unknown: no value, and the fill is drawn partway along.
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let fraction = (!self.indeterminate).then(|| {
            let span = (self.max - self.min).max(f32::EPSILON);
            ((self.value.unwrap_or(self.min) - self.min) / span).clamp(0.0, 1.0)
        });
        let value_text = match (&self.value_label, fraction) {
            _ if self.hide_label => None,
            (Some(l), _) => Some(l.clone()),
            (None, Some(f)) => Some(format!("{}%", (f * 100.0).round())),
            (None, None) => None,
        };
        let name = self.label.clone().unwrap_or_else(|| "Progress".to_owned());
        let mut bar = Bar {
            key: self.key,
            size: self.size,
            label: if self.hide_label { None } else { self.label },
            value_text: value_text.clone(),
            help: None,
            width: self.width,
            fraction,
            fill: t.accent_content_color_default,
            role_label: name,
        }
        .build(t);
        if let Some(f) = fraction {
            bar = bar.range_value(f64::from(f * 100.0), 0.0, 100.0);
        }
        if let Some(v) = value_text {
            bar = bar.a11y_value(v);
        }
        bar
    }
}
