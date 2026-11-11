//! Spectrum Action-button, from `design-data` visuals/action-button.json (container > icon,
//! progress, label, hold-icon, plus the keyboard focus ring). Every size, colour, radius and type
//! value comes from [`Tokens`].
//!
//! Where the oracle (`design-data render action-button`, see oracle.toml) looks odd, the button renders what
//! it renders and says so here:
//! - `isEmphasized` only matters together with `isSelected` (an advisory constraint): selected and
//!   emphasized is accent blue, emphasized alone is the default look.
//! - A static colour ignores `quiet` (the fill is always translucent), and a selected + emphasized
//!   static button loses its accent fill on hover and down.
//! - The hold icon keeps the *default* content colour on hover, down and on an unselected static
//!   button (dark on a static-white button), as the oracle draws it.
//! - Pending always draws the `gray_100` fill (no hover or down colour), quiet or not, and the
//!   progress is `accent_content_color_default` on a `gray_300` track. Static colour does nothing.
//! - Disabled ignores the static colour: an opaque `disabled_background_color` pill (transparent
//!   when quiet) with `disabled_content_color`, the same as a plain disabled button, so it is a
//!   grey pill on a coloured backdrop. Button's recipe has translucent static + disabled rules
//!   and this one does not; this is kept as rendered, not borrowed from Button (a14.23, see
//!   `disabled_ignores_the_static_colour`).
//! - The focus ring is `focus_indicator_color` for every static colour.
//! - Keyboard focus takes the hover fill of a selected button besides the ring (`El::focus_fill`:
//!   opaque, under the content, clipped to the pill, never from pointer focus; neutral 41 to 19,
//!   emphasized accent 59,99,251 to 39,77,234, quiet or not; a14.35). Unselected buttons and static
//!   colours keep their fill, as the oracle does, except that a static selected + emphasized one
//!   takes the accent hover colour. Icon and label colours do not change in focus. Disabled is not
//!   focusable. Not drawn: hover + keyboard-focus of an unselected button, where the oracle's focus
//!   wins over hover (ink `41`, not hover's `19`; a non-quiet fill `233`, not hover's `225`), because
//!   the view cannot see focus (a14.31); the hover look is drawn there.
//!
//! A selected button is a toggle: `.selected(..)` marks it as one (role `button`, `checked` =
//! ARIA `aria-pressed`). `on_press` only reports the press; the app flips `selected`.

use crate::button::{press_scale, StaticColor};
use crate::constraints::{self, OptionValue};
use crate::font;
use crate::icons::{
    CORNER_TRIANGLE_100, CORNER_TRIANGLE_200, CORNER_TRIANGLE_300, CORNER_TRIANGLE_75,
};
use crate::interaction::{Interaction, State};
use crate::size::ActionButtonSize;
use crate::tokens::Tokens;
use rune::tok::motion::SNAPPY;
use rune::{icon, row, text, Color, El, Icon};
use rune_kit::{Kit, Loader};

/// The hold icon's drawn size: the recipe sets none and the scene draws 6px at every size (the
/// `corner_triangle_icon_size_*` tokens give 5 to 8, which the oracle does not use).
const HOLD_ICON_SIZE: f32 = 6.0;

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    icon: f32,
    /// Container padding next to an icon, the progress or a label.
    pad: f32,
    /// Container padding when the label is hidden.
    pad_icon_only: f32,
    radius: f32,
    /// Distance of the hold icon from the bottom and end edges.
    hold_inset: f32,
    hold_glyph: &'static Icon,
}

impl ActionButtonSize {
    fn dims(self, t: &Tokens) -> Dims {
        use ActionButtonSize::*;
        match self {
            XS => Dims {
                height: t.component_height_50,
                gap: t.base_gap_extra_small,
                icon: t.workflow_icon_extra_small,
                pad: t.base_padding_horizontal_extra_small,
                pad_icon_only: t.base_padding_vertical_extra_small,
                radius: t.corner_radius_medium_size_extra_small,
                hold_inset: t.accessory_gap_affixed_extra_small,
                hold_glyph: &CORNER_TRIANGLE_75,
            },
            S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                icon: t.workflow_icon_small,
                pad: t.base_padding_horizontal_small,
                pad_icon_only: t.base_padding_vertical_small,
                radius: t.corner_radius_medium_size_small,
                hold_inset: t.accessory_gap_affixed_small,
                hold_glyph: &CORNER_TRIANGLE_75,
            },
            M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                icon: t.workflow_icon_medium,
                pad: t.base_padding_horizontal_medium,
                pad_icon_only: t.base_padding_vertical_medium,
                radius: t.corner_radius_medium_size_medium,
                hold_inset: t.accessory_gap_affixed_medium,
                hold_glyph: &CORNER_TRIANGLE_100,
            },
            L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                icon: t.workflow_icon_large,
                pad: t.base_padding_horizontal_large,
                pad_icon_only: t.base_padding_vertical_large,
                radius: t.corner_radius_medium_size_large,
                hold_inset: t.accessory_gap_affixed_large,
                hold_glyph: &CORNER_TRIANGLE_200,
            },
            XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                icon: t.workflow_icon_extra_large,
                pad: t.base_padding_horizontal_extra_large,
                pad_icon_only: t.base_padding_vertical_extra_large,
                radius: t.corner_radius_medium_size_extra_large,
                hold_inset: t.accessory_gap_affixed_extra_large,
                hold_glyph: &CORNER_TRIANGLE_300,
            },
        }
    }
}

/// Container fill (`None` is transparent) and the icon/label and hold-icon colours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Look {
    pub fill: Option<Color>,
    pub content: Color,
    pub hold: Color,
}

/// The builder state the colours depend on.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Flags {
    pub quiet: bool,
    pub selected: bool,
    pub emphasized: bool,
    pub static_color: Option<StaticColor>,
    pub disabled: bool,
    pub pending: bool,
}

fn by_state<T>(s: State, rest: T, hover: T, down: T) -> T {
    match s {
        State::Rest => rest,
        State::Hover => hover,
        State::Down => down,
    }
}

/// The (quiet, selected, emphasized, static) x (rest, hover, down, disabled, pending) colour table,
/// as the oracle renders it. Disabled and pending ignore the pointer state.
pub(crate) fn look(t: &Tokens, f: Flags, state: State) -> Look {
    if f.pending {
        // The recipe fills a pending button the same in every state, quiet included.
        let transparent = f.disabled && f.quiet;
        return Look {
            fill: (!transparent).then_some(t.gray_100),
            content: t.disabled_content_color,
            hold: t.disabled_content_color,
        };
    }
    if f.disabled {
        return Look {
            fill: (!f.quiet).then_some(t.disabled_background_color),
            content: t.disabled_content_color,
            hold: t.disabled_content_color,
        };
    }
    let accent = f.selected && f.emphasized;
    let content = if accent {
        t.white
    } else if f.selected {
        t.gray_25
    } else {
        by_state(
            state,
            t.neutral_content_color_default,
            t.neutral_content_color_hover,
            t.neutral_content_color_down,
        )
    };
    if let Some(c) = f.static_color {
        let (c100, c200, c800) = match c {
            StaticColor::White => (
                t.transparent_white_100,
                t.transparent_white_200,
                t.transparent_white_800,
            ),
            StaticColor::Black => (
                t.transparent_black_100,
                t.transparent_black_200,
                t.transparent_black_800,
            ),
        };
        return if accent {
            Look {
                fill: Some(by_state(
                    state,
                    t.accent_background_color_default,
                    c200,
                    c200,
                )),
                content,
                hold: content,
            }
        } else {
            Look {
                fill: Some(by_state(state, c100, c200, c200)),
                content: c800,
                hold: if f.selected {
                    t.gray_25
                } else {
                    t.neutral_content_color_default
                },
            }
        };
    }
    if accent {
        Look {
            fill: Some(by_state(
                state,
                t.accent_background_color_default,
                t.accent_background_color_hover,
                t.accent_background_color_down,
            )),
            content,
            hold: content,
        }
    } else if f.selected {
        Look {
            fill: Some(by_state(
                state,
                t.neutral_background_color_selected_default,
                t.neutral_background_color_selected_hover,
                t.neutral_background_color_selected_down,
            )),
            content,
            hold: content,
        }
    } else {
        let wash = by_state(state, t.gray_100, t.gray_200, t.gray_200);
        Look {
            fill: (!f.quiet || state != State::Rest).then_some(wash),
            content,
            hold: t.neutral_content_color_default,
        }
    }
}

type Press = Box<dyn FnOnce(El) -> El>;

pub struct ActionButton {
    key: String,
    label: Option<String>,
    size: ActionButtonSize,
    icon: Option<&'static Icon>,
    hide_label: bool,
    quiet: bool,
    /// `Some` marks a toggle: the value is its on/off state.
    selected: Option<bool>,
    emphasized: bool,
    pending: bool,
    disabled: bool,
    hold_icon: bool,
    static_color: Option<StaticColor>,
    press: Option<Press>,
}

impl ActionButton {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `icon`, `label`, `progress`, `hold-icon`).
    pub fn new(key: impl Into<String>) -> Self {
        ActionButton {
            key: key.into(),
            label: None,
            size: ActionButtonSize::M,
            icon: None,
            hide_label: false,
            quiet: false,
            selected: None,
            emphasized: false,
            pending: false,
            disabled: false,
            hold_icon: false,
            static_color: None,
            press: None,
        }
    }
    /// The visible label, and the accessible name even when hidden or pending.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// A shared [`Size`] or an [`ActionButtonSize`] (which adds `XS`).
    pub fn size(mut self, s: impl Into<ActionButtonSize>) -> Self {
        self.size = s.into();
        self
    }
    /// Any icon the app supplies; it is tinted with the label colour and sized by the recipe.
    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }
    /// An icon-only button. Needs an icon (nothing else would be drawn) and a label (the
    /// accessible name).
    pub fn hide_label(mut self, v: bool) -> Self {
        self.hide_label = v;
        self
    }
    /// No fill at rest.
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }
    /// Makes this a toggle button and sets its state (exposed as `checked`, ARIA `aria-pressed`).
    /// The press only reports; flip this in `on_press`. Leave it unset for a plain action.
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = Some(v);
        self
    }
    /// Accent colour when selected; no effect on an unselected button (advisory constraint).
    pub fn emphasized(mut self, v: bool) -> Self {
        self.emphasized = v;
        self
    }
    /// Replaces the icon, label and hold icon with a progress indicator. The button keeps reacting
    /// to the pointer (press scale), but `on_press` no longer fires, so a second click cannot
    /// submit twice. The catalog's 1 s delay before showing pending is the app's to implement.
    pub fn pending(mut self, v: bool) -> Self {
        self.pending = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// The corner triangle that says a popover opens on hold. The popover is the app's.
    pub fn hold_icon(mut self, v: bool) -> Self {
        self.hold_icon = v;
        self
    }
    /// For use on coloured backgrounds; the same values as [`crate::button::StaticColor`].
    pub fn static_color(mut self, c: StaticColor) -> Self {
        self.static_color = Some(c);
        self
    }
    /// Called on press (and Space or Enter).
    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e| e.on_press(f)));
        self
    }

    /// The builder state under the catalog's option names (`selectedTextColor` is not visual and
    /// has no builder method).
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        let mut o = vec![
            ("hideLabel", OptionValue::Bool(self.hide_label)),
            ("isQuiet", OptionValue::Bool(self.quiet)),
            ("isSelected", OptionValue::Bool(self.selected == Some(true))),
            ("isEmphasized", OptionValue::Bool(self.emphasized)),
            ("hasHoldIcon", OptionValue::Bool(self.hold_icon)),
            ("isPending", OptionValue::Bool(self.pending)),
            ("isDisabled", OptionValue::Bool(self.disabled)),
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
        constraints::assert_valid("action-button", &self.options());
        // Not a catalog constraint (accessible names belong to the accessibility spec).
        debug_assert!(
            !self.hide_label || self.label.is_some(),
            "an action button with a hidden label needs a label (its accessible name)"
        );
        let d = self.size.dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let look = look(
            t,
            Flags {
                quiet: self.quiet,
                selected: self.selected == Some(true),
                emphasized: self.emphasized,
                static_color: self.static_color,
                disabled: self.disabled,
                pending: self.pending,
            },
            state,
        );

        let show_label = !self.pending && !self.hide_label && self.label.is_some();
        let show_icon = !self.pending && self.icon.is_some();
        let pad = if self.hide_label {
            d.pad_icon_only
        } else {
            d.pad
        };
        let mut label_type = self.size.label_type(t);
        label_type.weight = t.medium_font_weight;

        let mut root = row()
            .key(part("container"))
            .h(d.height)
            .px(pad)
            .gap(d.gap)
            .center()
            .radius(d.radius)
            .fill(look.fill.unwrap_or(Color::TRANSPARENT))
            .token_motion(SNAPPY)
            .role("button")
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            );
        // Keyboard focus takes the hover fill of a selected button (the oracle's keyboard-focus
        // state), laid over its fill by Rune. Unselected buttons and static colours keep theirs
        // (translucent hover fills would stack, not replace), except that a static selected +
        // emphasized one goes from accent to the accent hover colour.
        if !self.disabled && !self.pending && self.selected == Some(true) {
            let focus = match (self.emphasized, self.static_color) {
                (true, _) => Some(t.accent_background_color_hover),
                (false, None) => Some(t.neutral_background_color_selected_hover),
                (false, Some(_)) => None,
            };
            if let Some(f) = focus {
                root = root.focus_fill(f);
            }
        }
        if let Some(on) = self.selected {
            root = root.checked(on);
        }
        if let Some(l) = &self.label {
            root = root.label(l.clone());
        }

        if self.pending {
            // Loader has one ink and draws its track as that ink at 0.2, not the recipe's gray_300.
            let spinner = Loader::new()
                .size(d.icon)
                .color(t.accent_content_color_default)
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
        if self.hold_icon && !self.pending {
            root = root.child(
                icon(d.hold_glyph, HOLD_ICON_SIZE)
                    .key(part("hold-icon"))
                    .color(look.hold)
                    .a11y_hidden()
                    .pin(None, None, Some(d.hold_inset), Some(d.hold_inset)),
            );
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
    use crate::size::Size;
    use crate::tokens::{tokens, ColorScheme, Scale};
    use serde_json::Value;

    fn rgba(s: &str) -> [f32; 4] {
        let v: Vec<f32> = s.split(',').map(|c| c.parse().unwrap()).collect();
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

    #[test]
    fn disabled_ignores_the_static_colour() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for static_color in [StaticColor::White, StaticColor::Black] {
            for quiet in [false, true] {
                let flags = |sc| Flags {
                    quiet,
                    disabled: true,
                    static_color: sc,
                    ..Flags::default()
                };
                let plain = look(t, flags(None), State::Rest);
                for state in [State::Rest, State::Hover, State::Down] {
                    assert_eq!(look(t, flags(Some(static_color)), state), plain);
                }
            }
        }
    }

    /// Every oracle case against the colour table: fill, icon/label colour and hold-icon colour
    /// (the built element exposes the fill and text colour, but this reaches the hold icon too).
    #[test]
    fn look_table_matches_oracle() {
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../tests/fixtures/action_button_scene.json"))
                .unwrap();
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
            let flag = |k: &str| c[k].as_bool().unwrap_or(false);
            let f = Flags {
                quiet: flag("quiet"),
                selected: flag("selected"),
                emphasized: flag("emphasized"),
                static_color: match c["static"].as_str() {
                    Some("white") => Some(StaticColor::White),
                    Some("black") => Some(StaticColor::Black),
                    _ => None,
                },
                disabled: flag("disabled"),
                pending: flag("pending"),
            };
            let state = match c["state"].as_str().unwrap() {
                "hover" => State::Hover,
                "down" => State::Down,
                _ => State::Rest,
            };
            let l = look(t, f, state);
            let what = format!("{c}");
            let what = &what[..what.len().min(160)];
            assert_color(l.fill, &c["container"]["fill"], what);
            for part in ["icon", "label_part"] {
                if !c[part].is_null() {
                    assert_color(Some(l.content), &c[part]["color"], what);
                }
            }
            if !c["hold"].is_null() {
                assert_color(Some(l.hold), &c["hold"]["color"], what);
            }
        }
    }

    #[test]
    fn oracle_cases_have_no_error_constraint_violations() {
        use crate::constraints::{check, Severity};
        let cases: Vec<Value> =
            serde_json::from_str(include_str!("../tests/fixtures/action_button_scene.json"))
                .unwrap();
        for c in &cases {
            let flag = |k: &str| OptionValue::Bool(c[k].as_bool().unwrap_or(false));
            let mut o = vec![
                ("hideLabel", flag("hidden")),
                ("isQuiet", flag("quiet")),
                ("isSelected", flag("selected")),
                ("isEmphasized", flag("emphasized")),
                ("hasHoldIcon", flag("hold_icon")),
                ("isPending", flag("pending")),
                ("isDisabled", flag("disabled")),
                ("icon", OptionValue::Present),
            ];
            if let Some(s) = c["static"].as_str() {
                o.push(("staticColor", OptionValue::Str(s)));
            }
            assert!(
                check("action-button", &o)
                    .iter()
                    .all(|v| v.severity != Severity::Error),
                "{c}"
            );
        }
    }

    #[test]
    fn xs_is_action_button_only_and_shared_sizes_convert() {
        assert_eq!(ActionButtonSize::from(Size::M), ActionButtonSize::M);
        assert_eq!(ActionButtonSize::from(Size::XL), ActionButtonSize::XL);
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        assert_eq!(ActionButtonSize::XS.dims(t).height, 20.0);
        assert_eq!(ActionButtonSize::XS.dims(t).radius, 6.0);
    }
}
