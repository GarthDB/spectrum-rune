//! Spectrum Radio-button, from `design-data` visuals/radio-button.json (container > control (>
//! selection-indicator), label, focus ring). Every size, colour, radius and type value comes from
//! [`Tokens`]. A radio only selects; choosing one of several is the job of a group (not built yet).
//!
//! Verification: geometry and fill against the oracle (`tests/radio_button.rs`); border and focus-ring colour,
//! width, thickness and gap from pixels, against the oracle's values (`tests/radio_button_pixels.rs`). The
//! font family is not asserted (not observable); keyboard-focus colours are not implemented (a14.14).

use crate::font;
use crate::interaction::{Interaction, State};
use crate::size::Size;
use crate::tokens::Tokens;
use rune::tok::motion::SNAPPY;
use rune::{el, row, text, El};

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    control: f32,
}

impl Size {
    fn radio_dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                control: t.radio_button_control_size_small,
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                control: t.radio_button_control_size_medium,
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                control: t.radio_button_control_size_large,
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                control: t.radio_button_control_size_extra_large,
            },
        }
    }
}

type Press = Box<dyn FnOnce(El) -> El>;

pub struct RadioButton {
    key: String,
    label: Option<String>,
    selected: bool,
    size: Size,
    emphasized: bool,
    disabled: bool,
    press: Option<Press>,
}

impl RadioButton {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `control`, `selection-indicator`, `label`).
    pub fn new(key: impl Into<String>) -> Self {
        RadioButton {
            key: key.into(),
            label: None,
            selected: false,
            size: Size::M,
            emphasized: false,
            disabled: false,
            press: None,
        }
    }
    /// Without a label only the control is drawn and the radio has no accessible name: the
    /// caller must provide one (e.g. a surrounding group's label).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
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
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Called on press (and Space). A radio never deselects itself: set your own `selected` here.
    pub fn on_select<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e| e.on_press(f)));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let d = self.size.radio_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let pick = |rest, hover, down| match state {
            State::Rest => rest,
            State::Hover => hover,
            State::Down => down,
        };
        let neutral = pick(
            t.neutral_content_color_default,
            t.neutral_content_color_hover,
            t.neutral_content_color_down,
        );
        // Precedence follows the recipe's priorities: disabled > emphasized > selected. The
        // oracle has no selected-down colour: a pressed selected radio keeps the default fill.
        let fill = if !self.selected {
            t.gray_25
        } else if self.disabled {
            t.disabled_content_color
        } else if self.emphasized {
            pick(
                t.accent_content_color_default,
                t.accent_content_color_hover,
                t.accent_content_color_down,
            )
        } else {
            pick(
                t.neutral_content_color_selected_default,
                t.neutral_content_color_selected_hover,
                t.neutral_content_color_selected_default,
            )
        };
        let stroke = if self.disabled {
            t.disabled_content_color
        } else {
            neutral
        };
        let label_color = if self.disabled {
            t.disabled_content_color
        } else {
            neutral
        };

        // The recipe presses the control by size-difference over a minimum perspective (24 / 26 =
        // 0.923). Rune's `press_scale` would scale the container, so the control is scaled from the
        // interaction state instead (`pressTarget: control`).
        let press = t.component_size_minimum_perspective_down
            / (t.component_size_minimum_perspective_down - t.component_size_difference_down);
        let dot = t.radio_button_selection_indicator;
        let inset = (d.control - dot) / 2.0;
        // Spectrum has no motion tokens; Rune's SNAPPY spring stands in until it does.
        let indicator = el()
            .key(part("selection-indicator"))
            .size(dot, dot)
            .absolute(inset, inset)
            .radius(t.corner_radius_full * dot)
            .fill(t.gray_25)
            .opacity(if self.selected { 1.0 } else { 0.0 })
            .motion(SNAPPY);
        let mut control = el()
            .key(part("control"))
            .size(d.control, d.control)
            .radius(t.corner_radius_full * d.control)
            .fill(fill)
            .scale(if state == State::Down { press } else { 1.0 })
            .token_motion(SNAPPY)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(indicator);
        // The recipe drops the border when selected (the fill takes over).
        if !self.selected {
            control = control.border(t.border_width_200, stroke);
        }

        let mut root = row()
            .key(part("container"))
            .h(d.height)
            .gap(d.gap)
            .center_y()
            .role("radio")
            .checked(self.selected)
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .child(control);
        if let Some(l) = self.label {
            root = root.label(l.clone()).child(
                text(l)
                    .key(part("label"))
                    .color(label_color)
                    .a11y_hidden()
                    .font(font::text_style(&self.size.label_type(t))),
            );
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
