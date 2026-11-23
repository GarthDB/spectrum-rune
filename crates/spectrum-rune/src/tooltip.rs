//! Spectrum Tooltip: a short label on a small coloured surface with a tip pointing at its trigger, from `design-data`
//! visuals/tooltip.json (a container with a label, and a tip on the side that faces the trigger). Every size, radius,
//! colour and type value comes from [`Tokens`].
//!
//! Like [`crate::popover::Popover`], the tooltip is its own element: put it anywhere, or give it a trigger with
//! [`Tooltip::anchor`] (Rune's `El::anchored`; mount it at the root of the view after the trigger). Showing it is the app's:
//! [`Tooltip::bind`] wires a trigger so the pointer resting on it for 1.5 s shows the tooltip, and leaving hides it.
//!
//! Built: the variants (`neutral`, `informative`, `negative`), the four placements, `maxWidth` (the label wraps at it; not
//! in the oracle, which does not wrap), `offset`. Not built: `shouldFlip`, and showing it for keyboard focus (Rune's views
//! cannot see focus, a14.31), so a tooltip is reachable by pointer only.
//!
//! Verification: geometry, colours and tip against the oracle (`tests/tooltip.rs`).

use crate::font;
use crate::popover::tip_glyph;
use crate::tokens::{Tokens, Typography};
use rune::{col, el, icon, row, text, Align, El, Side};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Informative,
    Negative,
}

/// Which side of its trigger the tooltip sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// How long the pointer rests on a trigger before [`Tooltip::bind`] shows its tooltip (React Spectrum's default delay).
pub const SHOW_DELAY: Duration = Duration::from_millis(1500);

pub struct Tooltip {
    key: String,
    label: String,
    variant: Variant,
    placement: Placement,
    max_width: f32,
    offset: f32,
    anchor: Option<String>,
}

impl Tooltip {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/label` and `{key}/tip`.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Tooltip {
            key: key.into(),
            label: label.into(),
            variant: Variant::default(),
            placement: Placement::default(),
            max_width: 160.0,
            offset: 4.0,
            anchor: None,
        }
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    /// The width the label wraps at (the catalog's `maxWidth`, 160 by default).
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    /// The distance from the trigger to the tip (the catalog's `offset`, 4 by default).
    pub fn offset(mut self, o: f32) -> Self {
        self.offset = o;
        self
    }
    /// Places the tooltip beside the element keyed `trigger`, on the side `placement` names.
    pub fn anchor(mut self, trigger: impl Into<String>) -> Self {
        self.anchor = Some(trigger.into());
        self
    }

    /// Wires `trigger` to `show`: called with `true` once the pointer has rested on it for [`SHOW_DELAY`], and `false` when it
    /// leaves (or presses). The app keeps the flag and builds the tooltip while it is set.
    pub fn bind<S: 'static>(trigger: El, show: impl Fn(&mut S, bool) + 'static) -> El {
        trigger
            .on_hover_change(move |s: &mut S, inside, _| show(s, inside))
            .hover_delays(SHOW_DELAY, Duration::ZERO)
    }

    pub fn build(self, t: &Tokens) -> El {
        let fill = match self.variant {
            Variant::Neutral => t.neutral_background_color_default,
            Variant::Informative => t.informative_background_color_default,
            Variant::Negative => t.negative_background_color_default,
        };
        let size = t.font_size_75;
        let style = Typography {
            size,
            weight: t.regular_font_weight,
            line_height: size * t.line_height_100,
            ..t.component_s_regular
        };
        let container = row()
            .key(format!("{}/container", self.key))
            .min_h(t.base_height_small)
            .max_w(self.max_width)
            .pl(t.base_padding_horizontal_small)
            .pr(t.base_padding_horizontal_small)
            .py(t.base_padding_vertical_small)
            .center_y()
            .radius(t.corner_radius_500)
            .fill(fill)
            .role("tooltip")
            .label(self.label.clone())
            .child(
                text(self.label)
                    .key(format!("{}/label", self.key))
                    .font(font::text_style(&style))
                    .color(t.gray_25)
                    .wrap()
                    .shrink()
                    .a11y_hidden(),
            );
        let side = match self.placement {
            Placement::Top => Side::Top,
            Placement::Bottom => Side::Bottom,
            Placement::Left => Side::Left,
            Placement::Right => Side::Right,
        };
        let (w, h) = match side {
            Side::Top | Side::Bottom => (t.tooltip_tip_width, t.tooltip_tip_height),
            Side::Left | Side::Right => (t.tooltip_tip_height, t.tooltip_tip_width),
        };
        // The tip's glyph fills one half of a square twice its width.
        let tip = el()
            .key(format!("{}/tip", self.key))
            .w(w)
            .h(h)
            .self_center()
            .child(icon(tip_glyph(side), t.tooltip_tip_width * 2.0).color(fill));
        let along_row = matches!(side, Side::Left | Side::Right);
        let mut root = if along_row { row() } else { col() };
        if matches!(side, Side::Bottom | Side::Right) {
            root = root.child(tip).child(container);
        } else {
            root = root.child(container).child(tip);
        }
        let mut root = crate::hug(root);
        if let Some(trigger) = self.anchor {
            // The tip fills the gap up to the offset.
            let gap = (self.offset - t.tooltip_tip_height).max(0.0);
            root = root.anchored(trigger, side, Align::Center, gap);
        }
        root
    }
}
