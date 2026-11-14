//! Spectrum Badge, from `design-data` visuals/badge.json (container > icon, label). A badge is
//! static: no hover, press or focus, so there is no `Interaction`, no handler and nothing focusable
//! (catalog accessibility: `focusable: false`, no role). Every size, colour, radius and type value
//! comes from [`Tokens`].
//!
//! What the oracle (`design-data render badge`, see oracle.toml) renders, followed as is:
//! - All 25 variants have their own bold and subtle fill (some are aliases of one colour: neutral
//!   and gray, informative, accent and blue, positive and green, negative and red, notice and
//!   orange).
//! - The recipe's outline rules name only six variants (neutral, informative, positive, negative,
//!   accent, notice). The other 19 render **outline as bold**: a solid fill, no stroke. Nothing is
//!   empty, but those 19 outline badges are not outlines (spectrum-design-data #1693).
//! - Bold labels are static white, except on yellow, notice, orange, chartreuse and celery, where
//!   they are static black. Subtle and outline labels are `gray_1000`.
//! - The radius is the same on both scales; the height, padding and type grow on mobile. The Spectrum 1 `fixed`
//!   option (square corners on one edge) is gone from the catalog (spectrum-design-data #1721), so it is gone here.
//! - Padding is the scene's `base-padding-horizontal-*` (not the recipe's `component-edge-to-text-*`),
//!   and `base-padding-vertical-*` for an icon-only badge, as for Action-button.

use crate::constraints::{self, OptionValue};
use crate::font;
use crate::size::Size;
use crate::tokens::Tokens;
use crate::wrap::Fit;
use rune::{icon, row, text, Color, El, Icon};

macro_rules! names {
    ($ty:ident { $($v:ident => $s:literal),* $(,)? }) => {
        impl $ty {
            /// The catalog's option value.
            pub fn as_str(self) -> &'static str {
                match self { $($ty::$v => $s),* }
            }
            /// Every value, in catalog order.
            pub const ALL: &'static [$ty] = &[$($ty::$v),*];
        }
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Informative,
    Positive,
    Negative,
    Indigo,
    Yellow,
    Magenta,
    Fuchsia,
    Purple,
    Seafoam,
    Accent,
    Notice,
    Gray,
    Red,
    Orange,
    Chartreuse,
    Celery,
    Green,
    Cyan,
    Blue,
    Pink,
    Turquoise,
    Brown,
    Cinnamon,
    Silver,
}

names!(Variant {
    Neutral => "neutral", Informative => "informative", Positive => "positive",
    Negative => "negative", Indigo => "indigo", Yellow => "yellow", Magenta => "magenta",
    Fuchsia => "fuchsia", Purple => "purple", Seafoam => "seafoam", Accent => "accent",
    Notice => "notice", Gray => "gray", Red => "red", Orange => "orange",
    Chartreuse => "chartreuse", Celery => "celery", Green => "green", Cyan => "cyan",
    Blue => "blue", Pink => "pink", Turquoise => "turquoise", Brown => "brown",
    Cinnamon => "cinnamon", Silver => "silver",
});

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Bold,
    Subtle,
    Outline,
}

names!(Style { Bold => "bold", Subtle => "subtle", Outline => "outline" });

/// Container fill, stroke (`None`: no border) and the icon/label colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Look {
    pub fill: Color,
    pub stroke: Option<Color>,
    pub content: Color,
}

/// The `(bold fill, subtle fill, outline stroke)` tokens of a variant. Only the six semantic
/// variants have an outline stroke.
fn palette(t: &Tokens, v: Variant) -> (Color, Color, Option<Color>) {
    use Variant::*;
    match v {
        Neutral => (
            t.neutral_subdued_background_color_default,
            t.neutral_subtle_background_color_default,
            Some(t.neutral_visual_color),
        ),
        Informative => (
            t.informative_background_color_default,
            t.informative_subtle_background_color_default,
            Some(t.informative_visual_color),
        ),
        Positive => (
            t.positive_background_color_default,
            t.positive_subtle_background_color_default,
            Some(t.positive_visual_color),
        ),
        Negative => (
            t.negative_background_color_default,
            t.negative_subtle_background_color_default,
            Some(t.negative_visual_color),
        ),
        Accent => (
            t.accent_background_color_default,
            t.accent_subtle_background_color_default,
            Some(t.accent_visual_color),
        ),
        Notice => (
            t.notice_background_color_default,
            t.notice_subtle_background_color_default,
            Some(t.notice_visual_color),
        ),
        Indigo => (
            t.indigo_background_color_default,
            t.indigo_subtle_background_color_default,
            None,
        ),
        Yellow => (
            t.yellow_background_color_default,
            t.yellow_subtle_background_color_default,
            None,
        ),
        Magenta => (
            t.magenta_background_color_default,
            t.magenta_subtle_background_color_default,
            None,
        ),
        Fuchsia => (
            t.fuchsia_background_color_default,
            t.fuchsia_subtle_background_color_default,
            None,
        ),
        Purple => (
            t.purple_background_color_default,
            t.purple_subtle_background_color_default,
            None,
        ),
        Seafoam => (
            t.seafoam_background_color_default,
            t.seafoam_subtle_background_color_default,
            None,
        ),
        Gray => (
            t.gray_background_color_default,
            t.gray_subtle_background_color_default,
            None,
        ),
        Red => (
            t.red_background_color_default,
            t.red_subtle_background_color_default,
            None,
        ),
        Orange => (
            t.orange_background_color_default,
            t.orange_subtle_background_color_default,
            None,
        ),
        Chartreuse => (
            t.chartreuse_background_color_default,
            t.chartreuse_subtle_background_color_default,
            None,
        ),
        Celery => (
            t.celery_background_color_default,
            t.celery_subtle_background_color_default,
            None,
        ),
        Green => (
            t.green_background_color_default,
            t.green_subtle_background_color_default,
            None,
        ),
        Cyan => (
            t.cyan_background_color_default,
            t.cyan_subtle_background_color_default,
            None,
        ),
        Blue => (
            t.blue_background_color_default,
            t.blue_subtle_background_color_default,
            None,
        ),
        Pink => (
            t.pink_background_color_default,
            t.pink_subtle_background_color_default,
            None,
        ),
        Turquoise => (
            t.turquoise_background_color_default,
            t.turquoise_subtle_background_color_default,
            None,
        ),
        Brown => (
            t.brown_background_color_default,
            t.brown_subtle_background_color_default,
            None,
        ),
        Cinnamon => (
            t.cinnamon_background_color_default,
            t.cinnamon_subtle_background_color_default,
            None,
        ),
        Silver => (
            t.silver_background_color_default,
            t.silver_subtle_background_color_default,
            None,
        ),
    }
}

/// The (variant, style) colour table, as the oracle renders it.
pub(crate) fn look(t: &Tokens, v: Variant, s: Style) -> Look {
    let (bold, subtle, visual) = palette(t, v);
    match (s, visual) {
        (Style::Subtle, _) => Look {
            fill: subtle,
            stroke: None,
            content: t.gray_1000,
        },
        (Style::Outline, Some(stroke)) => Look {
            fill: t.background_layer_2_color,
            stroke: Some(stroke),
            content: t.gray_1000,
        },
        // Bold, and the 19 variants the recipe gives no outline (they render as bold).
        _ => {
            use Variant::*;
            let dark_ink = matches!(v, Yellow | Notice | Orange | Chartreuse | Celery);
            Look {
                fill: bold,
                stroke: None,
                content: if dark_ink {
                    t.transparent_static_black_1000
                } else {
                    t.transparent_static_white_1000
                },
            }
        }
    }
}

struct Dims {
    height: f32,
    gap: f32,
    icon: f32,
    pad: f32,
    pad_icon_only: f32,
    radius: f32,
}

impl Size {
    fn badge_dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                icon: t.workflow_icon_small,
                pad: t.base_padding_horizontal_small,
                pad_icon_only: t.base_padding_vertical_small,
                radius: t.corner_radius_medium_size_small,
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                icon: t.workflow_icon_medium,
                pad: t.base_padding_horizontal_medium,
                pad_icon_only: t.base_padding_vertical_medium,
                radius: t.corner_radius_medium_size_medium,
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                icon: t.workflow_icon_large,
                pad: t.base_padding_horizontal_large,
                pad_icon_only: t.base_padding_vertical_large,
                radius: t.corner_radius_medium_size_large,
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                icon: t.workflow_icon_extra_large,
                pad: t.base_padding_horizontal_extra_large,
                pad_icon_only: t.base_padding_vertical_extra_large,
                radius: t.corner_radius_medium_size_extra_large,
            },
        }
    }
}

pub struct Badge {
    key: String,
    label: Option<String>,
    variant: Variant,
    style: Style,
    size: Size,
    icon: Option<&'static Icon>,
    width: Option<f32>,
}

impl Badge {
    /// `key` prefixes the parts: `{key}/container`, `{key}/icon`, `{key}/label`.
    pub fn new(key: impl Into<String>) -> Self {
        Badge {
            key: key.into(),
            label: None,
            variant: Variant::default(),
            style: Style::default(),
            // The catalog's default (the oracle CLI renders `m` unless told otherwise).
            size: Size::S,
            icon: None,
            width: None,
        }
    }
    /// Request width in points, like the oracle's `--width`: the root is that wide and a label too
    /// long for it wraps (the root grows by a line per extra line). Without it the root hugs.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// The visible text. Without one the badge is icon-only (and has no accessible name).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        // An empty label is no label: the oracle draws `--label ""` icon-only.
        self.label = Some(l.into()).filter(|l| !l.is_empty());
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    /// `Outline` only exists for neutral, informative, positive, negative, accent and notice; the
    /// other variants render it as bold (see the module docs).
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// Any icon the app supplies; it is tinted with the label colour and sized by the recipe.
    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    /// The builder state under the catalog's option names.
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        let mut o = vec![
            ("variant", OptionValue::Str(self.variant.as_str())),
            ("style", OptionValue::Str(self.style.as_str())),
            (
                "size",
                OptionValue::Str(match self.size {
                    Size::S => "s",
                    Size::M => "m",
                    Size::L => "l",
                    Size::XL => "xl",
                }),
            ),
        ];
        if self.label.is_some() {
            o.push(("label", OptionValue::Present));
        }
        if self.icon.is_some() {
            o.push(("icon", OptionValue::Present));
        }
        o
    }

    /// The root comes back inside a plain `row()` wrapper (see `crate::hug`), so it hugs its
    /// content in a column and centres in a row. A modifier chained on the result (`.w_full()`,
    /// `.w(..)`, `.key(..)`, margins) applies to the wrapper, not to the pill: to size or find the
    /// pill use its `<key>/container` part.
    pub fn build(self, t: &Tokens) -> El {
        constraints::assert_valid("badge", &self.options());
        let d = self.size.badge_dims(t);
        let look = look(t, self.variant, self.style);
        let part = |name: &str| format!("{}/{name}", self.key);
        let mut label_type = self.size.label_type(t);
        label_type.weight = t.medium_font_weight;

        let pad = if self.label.is_some() {
            d.pad
        } else {
            d.pad_icon_only
        };
        let fit = Fit::new(
            self.label.as_deref(),
            font::text_style(&label_type),
            label_type.line_height,
            if self.icon.is_some() { d.icon } else { 0.0 },
            d.height,
            self.width.map(|w| {
                w - 2.0 * pad
                    - if self.icon.is_some() {
                        d.icon + d.gap
                    } else {
                        0.0
                    }
            }),
        );
        let mut root = fit
            .root(row())
            .key(part("container"))
            .h(fit.height)
            .px(pad)
            .gap(d.gap)
            .radius(d.radius)
            .fill(look.fill);
        if let Some(stroke) = look.stroke {
            root = root.border(t.border_width_200, stroke);
        }
        if let Some(i) = self.icon {
            root = root.child(
                fit.part(
                    icon(i, d.icon)
                        .key(part("icon"))
                        .color(look.content)
                        .a11y_hidden(),
                    d.icon,
                ),
            );
        }
        // The text is the accessible name; the container has no role of its own.
        if let Some(l) = self.label {
            let mut label = text(l)
                .key(part("label"))
                .color(look.content)
                .font(font::text_style(&label_type));
            if self.width.is_some() {
                label = label.wrap().shrink();
            }
            root = root.child(fit.part(label, label_type.line_height));
        }
        if let Some(w) = self.width {
            root = root.w(w);
        }
        crate::hug(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{tokens, ColorScheme, Scale};
    use serde_json::Value;

    fn cases() -> Vec<Value> {
        serde_json::from_str(include_str!("../tests/fixtures/badge_scene.json")).unwrap()
    }

    fn pick<T: Copy>(c: &Value, key: &str, all: &[T], name: fn(T) -> &'static str) -> T {
        let want = c[key].as_str().unwrap();
        *all.iter().find(|v| name(**v) == want).unwrap()
    }

    fn assert_color(got: Color, want: &Value, what: &str) {
        let s = want.as_str().unwrap();
        let w: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
        let hex = got.hex();
        for i in 0..3 {
            let c = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap();
            assert!((c as f32 - w[i]).abs() <= 3.0, "{what}: {hex} want {s}");
        }
        assert!(
            (got.a - w.get(3).copied().unwrap_or(1.0)).abs() < 0.02,
            "{what}: alpha"
        );
    }

    /// Every oracle case against the colour table (fill, stroke, icon and label colour) and that
    /// the corners are plain, in every scheme and scale. The stroke is not on `ElementInfo`, so this is where
    /// it is checked; `tests/badge_pixels.rs` reads it from pixels as well.
    #[test]
    fn look_table_and_corners_match_oracle() {
        let cs = cases();
        let mut table = std::collections::HashSet::new();
        for c in &cs {
            let t = tokens(
                if c["scheme"] == "dark" {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                if c["scale"] == "mobile" {
                    Scale::Mobile
                } else {
                    Scale::Desktop
                },
            );
            let what = c.to_string();
            let what = &what[..what.len().min(160)];
            let v = pick(c, "variant", Variant::ALL, Variant::as_str);
            let s = pick(c, "style", Style::ALL, Style::as_str);
            let l = look(t, v, s);
            assert_color(l.fill, &c["container"]["fill"], what);
            match (&c["container"]["stroke"], l.stroke) {
                (Value::Null, None) => {}
                (w, Some(g)) => assert_color(g, w, what),
                (w, g) => panic!("{what}: stroke got {g:?}, want {w}"),
            }
            for part in ["icon", "label_part"] {
                if !c[part].is_null() {
                    assert_color(l.content, &c[part]["color"], what);
                }
            }
            assert!(
                c["container"]["radii"].is_null(),
                "{what}: the oracle has per-corner radii again"
            );
            if c["size"] == "m" && c["icon"].is_object() {
                table.insert((
                    c["variant"].to_string(),
                    c["style"].to_string(),
                    c["scheme"].to_string(),
                    c["scale"].to_string(),
                ));
            }
        }
        // Block A: 25 variants x 3 styles x 4 corners.
        assert_eq!(table.len(), 300);
    }

    #[test]
    fn all_values_are_listed() {
        assert_eq!(Variant::ALL.len(), 25);
        assert_eq!(Style::ALL.len(), 3);
    }

    #[test]
    fn only_semantic_variants_have_an_outline() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for v in Variant::ALL {
            let semantic = matches!(
                v,
                Variant::Neutral
                    | Variant::Informative
                    | Variant::Positive
                    | Variant::Negative
                    | Variant::Accent
                    | Variant::Notice
            );
            assert_eq!(
                look(t, *v, Style::Outline).stroke.is_some(),
                semantic,
                "{v:?}"
            );
            assert_eq!(
                look(t, *v, Style::Outline) == look(t, *v, Style::Bold),
                !semantic
            );
        }
    }
}
