//! Spectrum Button, from `design-data` visuals/button.json (container > icon (optional content),
//! progress, label, focus ring). Every size, colour, radius and type value comes from [`Tokens`].
//!
//! Where the oracle (`design-data render button`, 0.20.0) looks incomplete, the button renders what
//! it renders and says so here, so nothing is invented:
//! - Accent and negative have no outline look: that combination is an `error` constraint in the
//!   catalog, so `build` panics in debug builds. Release builds render what the oracle does: white
//!   text on no fill and no border.
//! - `static_color` only changes primary and secondary; on accent and negative it changes nothing.
//! - The focus ring is white or black for primary and secondary with a static colour, and
//!   `focus_indicator_color` otherwise (static colour is a no-op on accent and negative).
//! - The oracle's pending progress is gray_800 on every variant, invisible on a primary fill. We
//!   draw it in the content colour instead (the one deliberate departure).

use crate::constraints::{self, OptionValue};
use crate::font;
use crate::interaction::{Interaction, State};
use crate::size::Size;
use crate::tokens::Tokens;
use rune::tok::motion::SNAPPY;
use rune::{icon, row, text, Color, El, Icon};
use rune_kit::{Kit, Loader};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Accent,
    Negative,
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Fill,
    Outline,
}

/// For use on coloured backgrounds. Only primary and secondary react to it (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticColor {
    White,
    Black,
}

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    icon: f32,
    /// Container padding next to an icon or the progress (the recipe's only padding).
    pad: f32,
    /// Container padding when the label is hidden.
    pad_icon_only: f32,
    /// Container padding next to a label with no icon: not in the oracle, which always draws the
    /// icon. Spectrum's half-height `component-pill-edge-to-text`.
    pad_text: f32,
}

impl Size {
    fn button_dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                icon: t.workflow_icon_small,
                pad: t.base_padding_horizontal_small,
                pad_icon_only: t.base_padding_vertical_small,
                pad_text: t.component_pill_edge_to_text_75,
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                icon: t.workflow_icon_medium,
                pad: t.base_padding_horizontal_medium,
                pad_icon_only: t.base_padding_vertical_medium,
                pad_text: t.component_pill_edge_to_text_100,
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                icon: t.workflow_icon_large,
                pad: t.base_padding_horizontal_large,
                pad_icon_only: t.base_padding_vertical_large,
                pad_text: t.component_pill_edge_to_text_200,
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                icon: t.workflow_icon_extra_large,
                pad: t.base_padding_horizontal_extra_large,
                pad_icon_only: t.base_padding_vertical_extra_large,
                pad_text: t.component_pill_edge_to_text_300,
            },
        }
    }
}

/// Container fill and border (`None` is transparent) and the icon/label colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Look {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub content: Color,
}

/// The translucent ink of a static colour, and the opposite ink the primary fill's content takes.
struct Ink {
    c100: Color,
    c300: Color,
    c400: Color,
    c800: Color,
    c900: Color,
    opposite_c900: Color,
}

impl StaticColor {
    fn ink(self, t: &Tokens) -> Ink {
        match self {
            StaticColor::White => Ink {
                c100: t.transparent_white_100,
                c300: t.transparent_white_300,
                c400: t.transparent_white_400,
                c800: t.transparent_white_800,
                c900: t.transparent_white_900,
                opposite_c900: t.transparent_black_900,
            },
            StaticColor::Black => Ink {
                c100: t.transparent_black_100,
                c300: t.transparent_black_300,
                c400: t.transparent_black_400,
                c800: t.transparent_black_800,
                c900: t.transparent_black_900,
                opposite_c900: t.transparent_white_900,
            },
        }
    }
}

fn by_state<T>(s: State, rest: T, hover: T, down: T) -> T {
    match s {
        State::Rest => rest,
        State::Hover => hover,
        State::Down => down,
    }
}

/// The variant x style x state colour table, as the oracle renders it. Disabled ignores the state.
pub(crate) fn look(
    t: &Tokens,
    variant: Variant,
    style: Style,
    static_color: Option<StaticColor>,
    disabled: bool,
    state: State,
) -> Look {
    let outline = style == Style::Outline;
    let ink = static_color
        .filter(|_| matches!(variant, Variant::Primary | Variant::Secondary))
        .map(|c| c.ink(t));
    // Outline buttons take a fill only while hovered or pressed.
    let wash = |hover, down| {
        outline
            .then(|| by_state(state, None, Some(hover), Some(down)))
            .flatten()
    };

    if disabled {
        let (bg, border, content) = match &ink {
            Some(i) => (i.c100, i.c300, i.c400),
            None => (t.gray_100, t.gray_300, t.gray_400),
        };
        return Look {
            fill: (!outline).then_some(bg),
            stroke: outline.then_some(border),
            content,
        };
    }
    match (variant, ink) {
        (Variant::Accent | Variant::Negative, _) => {
            let (d, h, p) = if variant == Variant::Accent {
                (
                    t.accent_background_color_default,
                    t.accent_background_color_hover,
                    t.accent_background_color_down,
                )
            } else {
                (
                    t.negative_background_color_default,
                    t.negative_background_color_hover,
                    t.negative_background_color_down,
                )
            };
            Look {
                fill: (!outline).then(|| by_state(state, d, h, p)),
                stroke: None,
                content: t.transparent_static_white_1000,
            }
        }
        (Variant::Primary, Some(i)) => Look {
            fill: if outline {
                wash(i.c100, i.c100)
            } else {
                Some(i.c800)
            },
            stroke: outline.then_some(i.c800),
            content: if outline { i.c900 } else { i.opposite_c900 },
        },
        (Variant::Secondary, Some(i)) => Look {
            fill: if outline {
                wash(i.c100, i.c100)
            } else {
                Some(i.c100)
            },
            stroke: outline.then_some(i.c300),
            content: i.c900,
        },
        (Variant::Primary, None) => {
            let strong = by_state(state, t.gray_800, t.gray_900, t.gray_900);
            Look {
                fill: if outline {
                    wash(t.gray_100, t.gray_200)
                } else {
                    Some(strong)
                },
                stroke: outline.then_some(strong),
                content: if outline { strong } else { t.gray_25 },
            }
        }
        (Variant::Secondary, None) => Look {
            fill: if outline {
                wash(t.gray_100, t.gray_200)
            } else {
                Some(by_state(state, t.gray_100, t.gray_200, t.gray_200))
            },
            stroke: outline.then(|| by_state(state, t.gray_300, t.gray_400, t.gray_400)),
            content: by_state(state, t.gray_800, t.gray_900, t.gray_900),
        },
    }
}

/// The focus ring colour: only primary and secondary react to a static colour.
pub(crate) fn focus_ring_color(
    t: &Tokens,
    variant: Variant,
    static_color: Option<StaticColor>,
) -> Color {
    match (variant, static_color) {
        (Variant::Primary | Variant::Secondary, Some(StaticColor::White)) => {
            t.static_white_focus_indicator_color
        }
        (Variant::Primary | Variant::Secondary, Some(StaticColor::Black)) => {
            t.static_black_focus_indicator_color
        }
        _ => t.focus_indicator_color,
    }
}

/// The recipe presses by size difference over a perspective that grows with the width
/// (`pressWidthRatio`, clamped to the min/max perspective), so wide buttons shrink less.
pub(crate) fn press_scale(t: &Tokens, width: f32) -> f32 {
    let p = (width * t.component_size_width_ratio_down).clamp(
        t.component_size_minimum_perspective_down,
        t.component_size_maximum_perspective_down,
    );
    p / (p - t.component_size_difference_down)
}

type Press = Box<dyn FnOnce(El) -> El>;

pub struct Button {
    key: String,
    label: Option<String>,
    variant: Variant,
    style: Style,
    size: Size,
    icon: Option<&'static Icon>,
    label_hidden: bool,
    disabled: bool,
    pending: bool,
    static_color: Option<StaticColor>,
    press: Option<Press>,
}

impl Button {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `icon`, `label`, `progress`).
    pub fn new(key: impl Into<String>) -> Self {
        Button {
            key: key.into(),
            label: None,
            variant: Variant::Accent,
            style: Style::Fill,
            size: Size::M,
            icon: None,
            label_hidden: false,
            disabled: false,
            pending: false,
            static_color: None,
            press: None,
        }
    }
    /// The visible label, and the accessible name even when hidden or pending.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
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
    /// An icon-only button. Needs an icon (nothing else would be drawn) and a label (the
    /// accessible name).
    pub fn label_hidden(mut self, v: bool) -> Self {
        self.label_hidden = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Replaces the icon and label with a progress indicator. The button keeps reacting to the
    /// pointer, as the oracle shows (hover/down colours and press scale), but `on_press` no longer
    /// fires, so a second click cannot submit twice. The catalog's 1 s delay before showing
    /// pending is the app's to implement.
    pub fn pending(mut self, v: bool) -> Self {
        self.pending = v;
        self
    }
    pub fn static_color(mut self, c: StaticColor) -> Self {
        self.static_color = Some(c);
        self
    }
    /// Called on press (and Space or Enter).
    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e| e.on_press(f)));
        self
    }

    /// The builder state under the catalog's option names. Unset options are left out and take
    /// their catalog default.
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        let mut o = vec![
            (
                "variant",
                OptionValue::Str(match self.variant {
                    Variant::Accent => "accent",
                    Variant::Negative => "negative",
                    Variant::Primary => "primary",
                    Variant::Secondary => "secondary",
                }),
            ),
            (
                "style",
                OptionValue::Str(match self.style {
                    Style::Fill => "fill",
                    Style::Outline => "outline",
                }),
            ),
            ("isLabelHidden", OptionValue::Bool(self.label_hidden)),
        ];
        if self.icon.is_some() {
            o.push(("icon", OptionValue::Present));
        }
        if let Some(c) = self.static_color {
            o.push((
                "staticColor",
                OptionValue::Str(match c {
                    StaticColor::White => "white",
                    StaticColor::Black => "black",
                }),
            ));
        }
        o
    }

    /// The root comes back inside a plain `row()` wrapper (see `crate::hug`), so it hugs its
    /// content in a column and centres in a row. A modifier chained on the result (`.w_full()`,
    /// `.w(..)`, `.key(..)`, margins) applies to the wrapper, not to the pill: to size or find the
    /// pill use its `<key>/container` part.
    pub fn build(self, t: &Tokens) -> El {
        constraints::assert_valid("button", &self.options());
        // Not a catalog constraint (accessible names belong to the accessibility spec).
        debug_assert!(
            !self.label_hidden || self.label.is_some(),
            "a button with a hidden label needs a label (its accessible name)"
        );
        let d = self.size.button_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let look = look(
            t,
            self.variant,
            self.style,
            self.static_color,
            self.disabled,
            state,
        );

        let show_label = !self.pending && !self.label_hidden && self.label.is_some();
        let show_icon = !self.pending && self.icon.is_some();
        let pad = if self.label_hidden {
            d.pad_icon_only
        } else if self.pending || show_icon {
            d.pad
        } else {
            d.pad_text
        };
        let mut label_type = self.size.label_type(t);
        label_type.weight = t.bold_font_weight;

        let mut root = row()
            .key(part("container"))
            .h(d.height)
            .px(pad)
            .gap(d.gap)
            .center()
            .radius(t.corner_radius_full * d.height)
            .fill(look.fill.unwrap_or(Color::TRANSPARENT))
            .token_motion(SNAPPY)
            .role("button")
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                focus_ring_color(t, self.variant, self.static_color),
            );
        if let Some(c) = look.stroke {
            root = root.border(t.border_width_200, c);
        }
        if let Some(l) = &self.label {
            root = root.label(l.clone());
        }

        if self.pending {
            // Deliberately not the oracle's gray_800 (spectrum-design-data #1693): that is invisible on
            // a primary fill. The content colour is readable on every fill. Loader has one ink; its
            // track is that ink at 0.2, not the recipe's gray_300 track.
            let spinner = Loader::new()
                .size(d.icon)
                .color(look.content)
                .build(&Kit::neutral(false))
                .key(part("progress"));
            root = root.child(spinner);
        }
        if show_icon {
            if let Some(i) = self.icon {
                root = root.child(
                    icon(i, d.icon)
                        .key(part("icon"))
                        .color(look.content)
                        .a11y_hidden(),
                );
            }
        }
        if show_label {
            if let Some(l) = &self.label {
                root = root.child(
                    text(l.clone())
                        .key(part("label"))
                        .color(look.content)
                        .a11y_hidden()
                        .font(font::text_style(&label_type)),
                );
            }
        }

        let k = {
            let mut width = 2.0 * pad;
            if self.pending {
                width += d.icon;
            } else {
                if show_icon {
                    width += d.icon;
                }
                if let (true, Some(l)) = (show_label, &self.label) {
                    let w = rune::text::measure(l, &[], font::text_style(&label_type), None).0;
                    width += w + if show_icon { d.gap } else { 0.0 };
                }
            }
            press_scale(t, width)
        };

        if self.disabled {
            return crate::hug(root);
        }
        let root = interaction.attach(root);
        let root = match (self.pending, self.press) {
            (false, Some(p)) => p(root),
            _ => root,
        };
        crate::hug(root.press_scale(k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{tokens, ColorScheme, Scale};
    use serde_json::Value;

    fn rgba(s: &str) -> [f32; 4] {
        let v: Vec<f32> = s
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')')
            .split(',')
            .map(|c| c.trim().parse().unwrap())
            .collect();
        [v[0], v[1], v[2], v.get(3).copied().unwrap_or(1.0)]
    }

    fn assert_color(got: Option<Color>, want: &Value, what: &str) {
        // The fixture drops a null fill; an explicit transparent one stays a string.
        let s = want.as_str().unwrap_or("transparent");
        let want = (s != "transparent").then(|| rgba(s));
        match (got, want) {
            (None, None) => {}
            (Some(g), Some(w)) => {
                let hex = g.hex();
                for i in 0..3 {
                    let c = i32::from_str_radix(&hex[1 + 2 * i..3 + 2 * i], 16).unwrap();
                    assert!((c as f32 - w[i]).abs() <= 3.0, "{what}: {hex} want {s}");
                }
                assert!((g.a - w[3]).abs() < 0.02, "{what}: alpha {} want {s}", g.a);
            }
            (g, w) => panic!("{what}: got {g:?}, want {w:?} ({s})"),
        }
    }

    /// The border colour is not exposed by `ElementInfo` (a14.3), so the whole table (fill, border,
    /// content) is checked against every oracle case here, where the table is reachable.
    #[test]
    fn look_table_matches_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../tests/fixtures/button_scene.json")).unwrap();
        for c in &cases {
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
            let variant = match c["variant"].as_str().unwrap() {
                "accent" => Variant::Accent,
                "negative" => Variant::Negative,
                "primary" => Variant::Primary,
                _ => Variant::Secondary,
            };
            let style = if c["style"] == "outline" {
                Style::Outline
            } else {
                Style::Fill
            };
            let sc = match c["static"].as_str() {
                Some("white") => Some(StaticColor::White),
                Some("black") => Some(StaticColor::Black),
                _ => None,
            };
            let state = match c["state"].as_str().unwrap() {
                "hover" => State::Hover,
                "down" => State::Down,
                _ => State::Rest,
            };
            let l = look(
                t,
                variant,
                style,
                sc,
                c["disabled"].as_bool().unwrap_or(false),
                state,
            );
            let what = format!("{c}");
            let what = &what[..what.len().min(160)];
            assert_color(l.fill, &c["container"]["fill"], what);
            assert_color(l.stroke, &c["container"]["stroke"], what);
            // Pending has no content to compare; the icon and label share the colour.
            for part in ["icon", "label_part"] {
                if !c[part].is_null() {
                    assert_color(Some(l.content), &c[part]["color"], what);
                }
            }
        }
    }

    #[test]
    fn oracle_cases_have_no_error_constraint_violations() {
        use crate::constraints::{check, Severity};
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../tests/fixtures/button_scene.json")).unwrap();
        // 264 before the matrix followed the catalog constraints: 40 accent/negative outline cases dropped.
        assert_eq!(cases.len(), 224);
        for c in &cases {
            let mut o = vec![
                ("variant", OptionValue::Str(c["variant"].as_str().unwrap())),
                ("style", OptionValue::Str(c["style"].as_str().unwrap())),
                ("icon", OptionValue::Present),
                ("isLabelHidden", OptionValue::Bool(c["hidden"] == true)),
            ];
            if let Some(s) = c["static"].as_str() {
                o.push(("staticColor", OptionValue::Str(s)));
            }
            assert!(
                check("button", &o)
                    .iter()
                    .all(|v| v.severity != Severity::Error),
                "{c}"
            );
        }
    }

    #[test]
    fn focus_ring_color_matches_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../tests/fixtures/button_focus_ring.json")).unwrap();
        assert_eq!(cases.len(), 36);
        for c in &cases {
            let t = tokens(
                if c["scheme"] == "dark" {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                },
                Scale::Desktop,
            );
            let variant = match c["variant"].as_str().unwrap() {
                "accent" => Variant::Accent,
                "negative" => Variant::Negative,
                "primary" => Variant::Primary,
                _ => Variant::Secondary,
            };
            let sc = match c["static"].as_str() {
                Some("white") => Some(StaticColor::White),
                Some("black") => Some(StaticColor::Black),
                _ => None,
            };
            assert_color(
                Some(focus_ring_color(t, variant, sc)),
                &c["ring"],
                &format!("{c}"),
            );
        }
    }

    #[test]
    fn press_scale_clamps_to_the_perspective_range() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        assert!((press_scale(t, 54.44) - 24.0 / 26.0).abs() < 1e-4);
        assert!((press_scale(t, 80.2) - 0.9304).abs() < 1e-3);
        assert!((press_scale(t, 202.56) - 0.9712).abs() < 1e-3);
    }
}
