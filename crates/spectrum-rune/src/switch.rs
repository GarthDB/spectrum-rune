//! Spectrum Switch, from `design-data` visuals/switch.json (container > track > handle, label,
//! focus ring). Every size, colour, radius and type value comes from [`Tokens`].
//!
//! Verification: geometry and fill against the oracle (`tests/switch.rs`); border and focus-ring colour,
//! width, thickness and gap from pixels, against the oracle's values (`tests/switch_pixels.rs`). The
//! font family is not asserted (not observable).
//!
//! Keyboard focus takes the oracle's hover look besides the ring (a14.32): `El::focus_fill` paints
//! the hover colour over the unselected handle and the selected track (which has no border: the
//! oracle's stroke equals its fill there), and `hover_group_focus` + `group_color` give the label
//! the hover ink. Not matched: the UNSELECTED track's stroke, which the oracle takes from the
//! content colour (41 light, 219 dark) to its hover colour (19, 242) and Rune cannot, as it has no
//! focus-visible border colour (a14.14). A disabled switch is not focusable and keeps its colours.

use crate::font;
use crate::interaction::{Interaction, State};
pub use crate::size::Size;
use crate::tokens::{Tokens, Typography};
use crate::wrap::Fit;
use rune::tok::motion::SNAPPY;
use rune::{el, row, text, El};

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    track_w: f32,
    track_h: f32,
    handle: f32,
    handle_selected: f32,
    label: Typography,
}

impl Size {
    fn dims(self, t: &Tokens) -> Dims {
        match self {
            Size::S => Dims {
                height: t.component_height_75,
                gap: t.base_gap_small,
                track_w: t.switch_control_width_small,
                track_h: t.switch_control_height_small,
                handle: t.switch_handle_size_small,
                handle_selected: t.switch_handle_selected_size_small,
                label: self.label_type(t),
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                track_w: t.switch_control_width_medium,
                track_h: t.switch_control_height_medium,
                handle: t.switch_handle_size_medium,
                handle_selected: t.switch_handle_selected_size_medium,
                label: self.label_type(t),
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                track_w: t.switch_control_width_large,
                track_h: t.switch_control_height_large,
                handle: t.switch_handle_size_large,
                handle_selected: t.switch_handle_selected_size_large,
                label: self.label_type(t),
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                track_w: t.switch_control_width_extra_large,
                track_h: t.switch_control_height_extra_large,
                handle: t.switch_handle_size_extra_large,
                handle_selected: t.switch_handle_selected_size_extra_large,
                label: self.label_type(t),
            },
        }
    }
}

type Press = Box<dyn FnOnce(El) -> El>;

pub struct Switch {
    key: String,
    label: Option<String>,
    selected: bool,
    size: Size,
    emphasized: bool,
    disabled: bool,
    hide_label: bool,
    width: Option<f32>,
    press: Option<Press>,
}

impl Switch {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `track`, `handle`, `label`...), so `rt.find("{key}/track")` works.
    pub fn new(key: impl Into<String>) -> Self {
        Switch {
            key: key.into(),
            label: None,
            selected: false,
            size: Size::M,
            emphasized: false,
            disabled: false,
            hide_label: false,
            width: None,
            press: None,
        }
    }
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
    /// The label stays for accessibility but is not drawn.
    pub fn hide_label(mut self, v: bool) -> Self {
        self.hide_label = v;
        self
    }
    /// Called on press; flip your own `selected` state here.
    pub fn on_toggle<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(Box::new(move |e| e.on_press(f)));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let d = self.size.dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };

        // Selected + emphasized switches use the accent content colour; everything else the
        // neutral one. Disabled replaces it. The recipe reads the same colour for the track
        // stroke, the unselected handle and the label.
        let by = |accent: bool, state: State| match (accent, state) {
            (false, State::Rest) => t.neutral_content_color_default,
            (false, State::Hover) => t.neutral_content_color_hover,
            (false, State::Down) => t.neutral_content_color_down,
            (true, State::Rest) => t.accent_content_color_default,
            (true, State::Hover) => t.accent_content_color_hover,
            (true, State::Down) => t.accent_content_color_down,
        };
        let accent = self.selected && self.emphasized;
        let content = if self.disabled {
            t.disabled_content_color
        } else {
            by(accent, state)
        };
        let label_color = if self.disabled {
            t.disabled_content_color
        } else {
            by(false, state)
        };
        let (track_fill, handle_fill) = if self.selected {
            (content, t.gray_25)
        } else {
            (t.gray_25, content)
        };

        let border = t.border_width_200;
        let hs = if self.selected {
            d.handle_selected
        } else {
            d.handle
        };
        // Offsets are from the track's outer edge (Rune measures absolute children from the
        // border box here, checked against the oracle), and the handle is centred vertically.
        let inset = (d.track_h - hs) / 2.0;
        let x = if self.selected {
            d.track_w - inset - hs
        } else {
            inset
        };
        // Spectrum has no motion tokens; Rune's SNAPPY spring stands in until it does.
        let handle = el()
            .key(part("handle"))
            .size(hs, hs)
            .absolute(x, inset)
            .radius(t.corner_radius_full * hs)
            .fill(handle_fill)
            .motion(SNAPPY);
        // Keyboard focus takes the hover look (the oracle's keyboard-focus state): `focus_fill`
        // paints the hover colour over the part whose fill is the content colour, the unselected
        // handle or the selected track. Not the unselected track's stroke: Rune has no
        // focus-visible border colour (a14.14).
        let handle = if self.disabled || self.selected {
            handle
        } else {
            handle.focus_fill(by(false, State::Hover))
        };
        let track = el()
            .key(part("track"))
            .size(d.track_w, d.track_h)
            .radius(t.corner_radius_full * d.track_h)
            .fill(track_fill)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(handle);
        // A selected track's stroke equals its fill in the oracle, so it has no border there: one
        // would keep its rest colour as a rim around the focus fill. (Borders are drawn inside the
        // box, so the layout is the same.)
        let track = if self.selected {
            if self.disabled {
                track
            } else {
                track.focus_fill(by(accent, State::Hover))
            }
        } else {
            track.border(border, content)
        };

        let shown = self.label.as_deref().filter(|_| !self.hide_label);
        let fit = Fit::new(
            shown,
            font::text_style(&d.label),
            d.label.line_height,
            d.track_h,
            d.height,
            self.width.map(|w| w - d.track_w - d.gap),
        );
        let mut root = fit
            .root(row())
            .key(part("container"))
            .h(fit.height)
            .gap(d.gap)
            .role("switch")
            .checked(self.selected)
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .hover_group_focus()
            .child(fit.part(track, d.track_h));
        if let Some(l) = self.label {
            root = root.label(l.clone());
            if !self.hide_label {
                let mut label = text(l)
                    .key(part("label"))
                    .color(label_color)
                    .a11y_hidden()
                    .font(font::text_style(&d.label));
                // The hover colour while the root shows keyboard focus (`hover_group_focus`). The
                // oracle's down colour equals its hover colour, so while a press is held the group
                // colour is the same as the rebuilt one (`tests/switch_pixels.rs`).
                if !self.disabled {
                    label = label.group_color(by(false, State::Hover));
                }
                if self.width.is_some() {
                    label = label.wrap().shrink();
                }
                root = root.child(fit.part(label, d.label.line_height));
            }
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
