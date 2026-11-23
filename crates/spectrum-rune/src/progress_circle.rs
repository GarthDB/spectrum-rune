//! Spectrum Progress circle: a ring that fills clockwise from the top as work completes, from `design-data`
//! visuals/progress-circle.json (a track ring and a fill arc). Every size, thickness and colour comes from [`Tokens`].
//!
//! Built: `value` with `min_value` / `max_value`, the sizes `S`, `M` and `L` (the recipe has no `XL`: it draws as `L`),
//! and `indeterminate`, drawn at rest as a quarter of the ring (the oracle gives it no sweep: it spins, which is not built). Not
//! built: `staticColor`. The arc's ends are square.
//!
//! Verification: size, stroke, sweep and colours against the oracle (`tests/progress_circle.rs`, the sweep from pixels).

use crate::tokens::Tokens;
pub use crate::Size;
use rune::{el, icon, El, Icon};
use std::cell::RefCell;
use std::collections::HashMap;

type Made = HashMap<(u32, u32, u32, bool), &'static Icon>;

/// The ring and arc icons, made once per (size, thickness, sweep in tenths of a percent): an `Icon` is `'static`.
fn glyph(size: f32, thickness: f32, sweep_tenths: u32, arc: bool) -> &'static Icon {
    thread_local! {
        static MADE: RefCell<Made> = RefCell::new(Made::new());
    }
    MADE.with(|m| {
        *m.borrow_mut()
            .entry((size.to_bits(), thickness.to_bits(), sweep_tenths, arc))
            .or_insert_with(|| {
                let (c, r) = (size / 2.0, (size - thickness) / 2.0);
                let body = if !arc || sweep_tenths >= 1000 {
                    format!(
                        r#"<circle cx="{c}" cy="{c}" r="{r}" stroke="currentColor" stroke-width="{thickness}" fill="none"/>"#
                    )
                } else if sweep_tenths == 0 {
                    String::new()
                } else {
                    let theta = sweep_tenths as f32 / 1000.0 * std::f32::consts::TAU;
                    let (x, y) = (c + r * theta.sin(), c - r * theta.cos());
                    let large = u8::from(sweep_tenths > 500);
                    format!(
                        r#"<path d="M{c} {} A{r} {r} 0 {large} 1 {x:.3} {y:.3}" stroke="currentColor" stroke-width="{thickness}" fill="none"/>"#,
                        c - r
                    )
                };
                Box::leak(Box::new(Icon {
                    name: "progress-circle",
                    view: size,
                    body: Box::leak(body.into_boxed_str()),
                }))
            })
    })
}

pub struct ProgressCircle {
    key: String,
    size: Size,
    label: Option<String>,
    value: Option<f32>,
    min: f32,
    max: f32,
    indeterminate: bool,
}

impl ProgressCircle {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/track` and `{key}/fill`.
    pub fn new(key: impl Into<String>) -> Self {
        ProgressCircle {
            key: key.into(),
            size: Size::M,
            label: None,
            value: None,
            min: 0.0,
            max: 100.0,
            indeterminate: false,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// The accessible name. Every progress circle should have one: it has no visible label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
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
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        debug_assert!(self.size != Size::XL, "the progress circle has no XL size");
        let (size, thickness) = match self.size {
            Size::S => (t.visual_size_50, t.progress_circle_thickness_small),
            Size::M => (t.visual_size_300, t.progress_circle_thickness_medium),
            Size::L | Size::XL => (t.visual_size_800, t.progress_circle_thickness_large),
        };
        let span = (self.max - self.min).max(f32::EPSILON);
        let fraction = if self.indeterminate {
            0.25
        } else {
            ((self.value.unwrap_or(self.min) - self.min) / span).clamp(0.0, 1.0)
        };
        let tenths = (fraction * 1000.0).round() as u32;
        let part = |name: &str| format!("{}/{name}", self.key);
        let mut root = el()
            .key(part("container"))
            .w(size)
            .h(size)
            .role("progressbar")
            .label(self.label.unwrap_or_else(|| "Progress".to_owned()))
            .child(
                icon(glyph(size, thickness, 0, false), size)
                    .key(part("track"))
                    .color(t.track_color)
                    .absolute(0.0, 0.0),
            )
            .child(
                icon(glyph(size, thickness, tenths, true), size)
                    .key(part("fill"))
                    .color(t.accent_content_color_default)
                    .absolute(0.0, 0.0),
            );
        if !self.indeterminate {
            root = root.range_value(f64::from(fraction * 100.0), 0.0, 100.0);
        }
        root
    }
}
