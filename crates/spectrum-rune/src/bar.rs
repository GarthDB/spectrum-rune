//! The bar both [`crate::progress_bar::ProgressBar`] and [`crate::meter::Meter`] draw: a header (label and value), a
//! track with a fill, and optional help text, from the `progress-bar` and `meter` recipes (they share one layout).

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{col, el, row, text, Color, El};

pub(crate) struct Bar {
    pub key: String,
    pub size: Size,
    pub label: Option<String>,
    pub value_text: Option<String>,
    pub help: Option<String>,
    pub width: Option<f32>,
    /// 0..=1. `None`: indeterminate (the fill is drawn where the oracle's frame has it, 25 % in).
    pub fraction: Option<f32>,
    pub fill: Color,
    pub role_label: String,
}

impl Bar {
    pub(crate) fn build(self, t: &Tokens) -> El {
        let (gap, font_size, thickness) = match self.size {
            Size::S => (t.base_gap_small, t.font_size_75, t.bar_thickness_small),
            Size::M => (t.base_gap_medium, t.font_size_100, t.bar_thickness_medium),
            Size::L => (t.base_gap_large, t.font_size_200, t.bar_thickness_large),
            Size::XL => (
                t.base_gap_extra_large,
                t.font_size_300,
                t.bar_thickness_extra_large,
            ),
        };
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let ink = t.neutral_subdued_content_color_default;
        let part = |name: &str| format!("{}/{name}", self.key);
        let line = |name: &str, s: &str| {
            text(s.to_owned())
                .key(part(name))
                .font(style)
                .color(ink)
                .a11y_hidden()
        };
        let mut root = col()
            .key(part("container"))
            .gap(gap)
            .min_w(t.bar_minimum_width_extra_large)
            .role("progressbar")
            .label(self.role_label.clone());
        root = match self.width {
            Some(w) => root.w(w),
            None => root,
        };
        if self.label.is_some() || self.value_text.is_some() {
            let mut header = row().key(part("header")).w_full().gap(t.spacing_200);
            header = header.child(match &self.label {
                Some(l) => line("label", l).grow(1.0),
                None => el().grow(1.0),
            });
            if let Some(v) = &self.value_text {
                header = header.child(line("value", v));
            }
            root = root.child(header);
        }
        let radius = thickness / 2.0;
        let (lead, share) = match self.fraction {
            Some(f) => (0.0, f.clamp(0.0, 1.0)),
            None => (0.25, 0.75),
        };
        let mut track = row()
            .key(part("track"))
            .w_full()
            .h(thickness)
            .radius(radius)
            .fill(t.track_color);
        if lead > 0.0 {
            track = track.child(el().w_frac(lead).h(thickness));
        }
        // An empty fill has no width to round or paint (the renderer rejects a zero-sized rounded box).
        let fill = el().key(part("fill")).w_frac(share).h(thickness);
        track = track.child(if share > 0.0 {
            fill.radius(radius).fill(self.fill)
        } else {
            fill
        });
        root = root.child(track);
        if let Some(h) = &self.help {
            root = root.child(line("help-text", h));
        }
        crate::hug(root)
    }
}
