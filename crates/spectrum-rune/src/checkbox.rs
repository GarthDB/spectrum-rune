//! Spectrum Checkbox, from `design-data` visuals/checkbox.json (container > box (> checkmark | dash),
//! label, focus ring). Every size, colour, radius and type value comes from [`Tokens`], except the
//! icon sizes, which the recipe itself writes as literals.
//!
//! Verification: geometry and fill against the oracle (`tests/checkbox.rs`); border and focus-ring colour,
//! width, thickness and gap from pixels, against the oracle's values (`tests/checkbox_pixels.rs`). The
//! font family is not asserted (not observable).
//!
//! Keyboard focus takes the oracle's hover look besides the ring (a14.33): `El::focus_fill` paints
//! the hover fill over a selected or indeterminate box (neutral, emphasized accent, error; a plain
//! indeterminate box keeps its fill), and `hover_group_focus` + `group_color` give the label the
//! hover ink. The oracle's box stroke is 0 wide on those, so only the fill matters. Not matched:
//! the UNSELECTED box's 2px stroke, from the content colour (41 light / 219 dark; error 215,50,32 /
//! 252,67,46) to its hover colour (19 / 242; error 183,40,24 / 255,103,86), which Rune cannot draw
//! (no focus-visible border colour, a14.14). A disabled checkbox is not focusable.

use crate::constraints::{self, OptionValue};
use crate::font;
use crate::icons::icon as ui_icon;
use crate::interaction::{Interaction, State};
use crate::tokens::{Tokens, Typography};
use crate::wrap::Fit;
use rune::tok::motion::SNAPPY;
use rune::{el, icon, row, text, El};

pub use crate::size::Size;

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    control: f32,
    radius: f32,
    /// Drawn size of the checkmark and dash: literal in the recipe (8/10/10/12).
    mark: f32,
    /// The `ui.json` icon size key the oracle uses: the size index (75/100/200/300).
    mark_set: u32,
    label: Typography,
}

impl Size {
    fn checkbox_dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                control: t.checkbox_control_size_small,
                radius: t.corner_radius_small_size_small,
                mark: 8.0,
                mark_set: 75,
                label: self.label_type(t),
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                control: t.checkbox_control_size_medium,
                radius: t.corner_radius_small_size_medium,
                mark: 10.0,
                mark_set: 100,
                label: self.label_type(t),
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                control: t.checkbox_control_size_large,
                radius: t.corner_radius_small_size_large,
                mark: 10.0,
                mark_set: 200,
                label: self.label_type(t),
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                control: t.checkbox_control_size_extra_large,
                radius: t.corner_radius_small_size_extra_large,
                mark: 12.0,
                mark_set: 300,
                label: self.label_type(t),
            },
        }
    }
}

type Press = Box<dyn FnOnce(El) -> El>;

pub struct Checkbox {
    key: String,
    label: Option<String>,
    selected: bool,
    indeterminate: bool,
    size: Size,
    emphasized: bool,
    error: bool,
    disabled: bool,
    width: Option<f32>,
    press: Option<Press>,
}

impl Checkbox {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `box`, `checkmark`, `dash`, `label`), so `rt.find("{key}/box")` works.
    pub fn new(key: impl Into<String>) -> Self {
        Checkbox {
            key: key.into(),
            label: None,
            selected: false,
            indeterminate: false,
            size: Size::M,
            emphasized: false,
            error: false,
            disabled: false,
            width: None,
            press: None,
        }
    }
    /// Without a label the checkbox is standalone: only the box is drawn.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Request width in points, like the oracle's `--width`: the root is that wide and a label too
    /// long for it wraps (the root grows by a line per extra line). Without it the root hugs.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    /// Shows the dash; wins over `selected`.
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn emphasized(mut self, v: bool) -> Self {
        self.emphasized = v;
        self
    }
    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Called on press; flip your own `selected` state here.
    pub fn on_toggle<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e| e.on_press(f)));
        self
    }

    /// The builder state under the catalog's option names. Unset options are left out and take
    /// their catalog default.
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        vec![
            ("isSelected", OptionValue::Bool(self.selected)),
            ("isIndeterminate", OptionValue::Bool(self.indeterminate)),
        ]
    }

    pub fn build(self, t: &Tokens) -> El {
        constraints::assert_valid("checkbox", &self.options());
        let d = self.size.checkbox_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let pick_at = |s, rest, hover, down| match s {
            State::Rest => rest,
            State::Hover => hover,
            State::Down => down,
        };
        let pick = |rest, hover, down| pick_at(state, rest, hover, down);
        let neutral = pick(
            t.neutral_content_color_default,
            t.neutral_content_color_hover,
            t.neutral_content_color_down,
        );
        let negative = pick(
            t.negative_content_color_default,
            t.negative_content_color_hover,
            t.negative_content_color_down,
        );

        let on = self.selected || self.indeterminate;
        // Precedence follows the recipe's priorities: disabled > error > emphasized > selected.
        // A plain indeterminate box keeps the default selected fill in every state (oracle).
        let fill_at = |s| {
            if !on {
                t.gray_25
            } else if self.disabled {
                t.disabled_content_color
            } else if self.error {
                pick_at(
                    s,
                    t.negative_content_color_default,
                    t.negative_content_color_hover,
                    t.negative_content_color_down,
                )
            } else if self.emphasized {
                pick_at(
                    s,
                    t.accent_content_color_default,
                    t.accent_content_color_hover,
                    t.accent_content_color_down,
                )
            } else if self.indeterminate {
                t.neutral_background_color_selected_default
            } else {
                pick_at(
                    s,
                    t.neutral_background_color_selected_default,
                    t.neutral_background_color_selected_hover,
                    t.neutral_background_color_selected_down,
                )
            }
        };
        let fill = fill_at(state);
        let stroke = if self.disabled {
            t.disabled_content_color
        } else if self.error {
            negative
        } else {
            neutral
        };
        let label_color = if self.disabled {
            t.disabled_content_color
        } else {
            neutral
        };

        // The recipe presses the box by size-difference over a minimum perspective: 24 / 26 =
        // 0.923 at every size. Rune's `press_scale` only scales the element that holds the press
        // (the container), so the box is scaled from the interaction state instead.
        let press = t.component_size_minimum_perspective_down
            / (t.component_size_minimum_perspective_down - t.component_size_difference_down);
        let inset = (d.control - d.mark) / 2.0;
        let mark = |name: &str, visible: bool| {
            let i = ui_icon(name, d.mark_set).expect("ui icon");
            icon(i, d.mark)
                .key(part(name))
                .absolute(inset, inset)
                .color(t.gray_25)
                .opacity(if visible { 1.0 } else { 0.0 })
        };
        // Spectrum has no motion tokens; Rune's SNAPPY spring stands in until it does.
        let mut control = el()
            .key(part("box"))
            .size(d.control, d.control)
            .radius(d.radius)
            .fill(fill)
            .scale(if state == State::Down { press } else { 1.0 })
            .token_motion(SNAPPY)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(mark("checkmark", self.selected && !self.indeterminate))
            .child(mark("dash", self.indeterminate));
        if !on {
            control = control.border(t.border_width_200, stroke);
        }
        // Keyboard focus takes the hover fill of a selected or indeterminate box (the oracle's
        // keyboard-focus state), laid over it by Rune. A plain indeterminate box keeps its fill.
        // The unselected box's stroke is not matched: no focus-visible border colour (a14.14).
        if on && !self.disabled && fill_at(State::Hover) != fill_at(State::Rest) {
            control = control.focus_fill(fill_at(State::Hover));
        }

        let fit = Fit::new(
            self.label.as_deref(),
            font::text_style(&d.label),
            d.label.line_height,
            d.control,
            d.height,
            self.width.map(|w| w - d.control - d.gap),
        );
        let mut root = fit
            .root(row())
            .key(part("container"))
            .h(fit.height)
            .gap(d.gap)
            .role("checkbox")
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .hover_group_focus()
            .child(fit.part(control, d.control));
        root = if self.indeterminate {
            root.checked_mixed()
        } else {
            root.checked(self.selected)
        };
        if let Some(l) = self.label {
            let mut label = text(l.clone())
                .key(part("label"))
                .color(label_color)
                .a11y_hidden()
                .font(font::text_style(&d.label));
            // The hover colour while the root shows keyboard focus (`hover_group_focus`); the
            // oracle's down colour equals its hover colour, so a held press is not overridden.
            if !self.disabled {
                label = label.group_color(t.neutral_content_color_hover);
            }
            if self.width.is_some() {
                label = label.wrap().shrink();
            }
            root = root.label(l).child(fit.part(label, d.label.line_height));
        }
        if let Some(w) = self.width {
            root = root.w(w);
        }
        if self.disabled {
            root
        } else {
            let root = interaction.attach(root);
            match self.press {
                Some(p) => p(root),
                None => root,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{tokens, ColorScheme, Scale};

    /// The oracle draws the mark at 8/10/10/12 points, from the icon whose size key is the size index
    /// (viewBox checkmark 10/10/12/14, dash 8/10/12/12).
    #[test]
    fn mark_icon_sizes_match_oracle() {
        let t = tokens(ColorScheme::Light, Scale::Desktop);
        for (size, drawn, check, dash) in [
            (Size::S, 8.0, 10.0, 8.0),
            (Size::M, 10.0, 10.0, 10.0),
            (Size::L, 10.0, 12.0, 12.0),
            (Size::XL, 12.0, 14.0, 12.0),
        ] {
            let d = size.checkbox_dims(t);
            assert_eq!(d.mark, drawn);
            assert_eq!(ui_icon("checkmark", d.mark_set).unwrap().view, check);
            assert_eq!(ui_icon("dash", d.mark_set).unwrap().view, dash);
        }
    }
}
