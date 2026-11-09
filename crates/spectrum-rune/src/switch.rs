//! Spectrum Switch, from `design-data` visuals/switch.json (container > track > handle, label,
//! focus ring). Every size, colour, radius and type value comes from [`Tokens`].

use crate::interaction::{Interaction, State};
use crate::tokens::Tokens;
use rune::tok::motion::SNAPPY;
use rune::{el, row, text, El, Family, TextStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    S,
    #[default]
    M,
    L,
    XL,
}

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    gap: f32,
    track_w: f32,
    track_h: f32,
    handle: f32,
    handle_selected: f32,
    font_size: f32,
    line_height: f32,
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
                font_size: t.font_size_75,
                line_height: t.line_height_font_size_75,
            },
            Size::M => Dims {
                height: t.component_height_100,
                gap: t.base_gap_medium,
                track_w: t.switch_control_width_medium,
                track_h: t.switch_control_height_medium,
                handle: t.switch_handle_size_medium,
                handle_selected: t.switch_handle_selected_size_medium,
                font_size: t.font_size_100,
                line_height: t.line_height_font_size_100,
            },
            Size::L => Dims {
                height: t.component_height_200,
                gap: t.base_gap_large,
                track_w: t.switch_control_width_large,
                track_h: t.switch_control_height_large,
                handle: t.switch_handle_size_large,
                handle_selected: t.switch_handle_selected_size_large,
                font_size: t.font_size_200,
                line_height: t.line_height_font_size_200,
            },
            Size::XL => Dims {
                height: t.component_height_300,
                gap: t.base_gap_extra_large,
                track_w: t.switch_control_width_extra_large,
                track_h: t.switch_control_height_extra_large,
                handle: t.switch_handle_size_extra_large,
                handle_selected: t.switch_handle_selected_size_extra_large,
                font_size: t.font_size_300,
                line_height: t.line_height_font_size_300,
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
    press: Option<Press>,
}

impl Switch {
    /// `key` must be unique per switch: it keys the pointer state. The part keys
    /// (`container`, `track`, `handle`, `label`) follow the recipe and repeat across switches.
    pub fn new(key: impl Into<String>) -> Self {
        Switch {
            key: key.into(),
            label: None,
            selected: false,
            size: Size::M,
            emphasized: false,
            disabled: false,
            hide_label: false,
            press: None,
        }
    }
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
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };

        // Selected + emphasized switches use the accent content colour; everything else the
        // neutral one. Disabled replaces it. The recipe reads the same colour for the track
        // stroke, the unselected handle and the label.
        let by_state = |accent: bool| match (accent, state) {
            (false, State::Rest) => t.neutral_content_color_default,
            (false, State::Hover) => t.neutral_content_color_hover,
            (false, State::Down) => t.neutral_content_color_down,
            (true, State::Rest) => t.accent_content_color_default,
            (true, State::Hover) => t.accent_content_color_hover,
            (true, State::Down) => t.accent_content_color_down,
        };
        let content = if self.disabled {
            t.disabled_content_color
        } else {
            by_state(self.selected && self.emphasized)
        };
        let label_color = if self.disabled {
            t.disabled_content_color
        } else {
            by_state(false)
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
            .key("handle")
            .size(hs, hs)
            .absolute(x, inset)
            .radius(t.corner_radius_full * hs)
            .fill(handle_fill)
            .motion(SNAPPY);
        let track = el()
            .key("track")
            .size(d.track_w, d.track_h)
            .radius(t.corner_radius_full * d.track_h)
            .fill(track_fill)
            .border(border, content)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(handle);

        let mut root = row()
            .key("container")
            .h(d.height)
            .gap(d.gap)
            .center_y()
            .role("switch")
            .checked(self.selected)
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .child(track);
        if let Some(l) = self.label {
            root = root.label(l.clone());
            if !self.hide_label {
                root = root.child(
                    text(l)
                        .key("label")
                        .color(label_color)
                        .a11y_hidden()
                        // The family fallback list (a14.5) is not built yet; this needs the font installed.
                        .font(
                            TextStyle::new(d.font_size, f32::from(t.regular_font_weight))
                                .family(Family::Named("Adobe Clean Spectrum VF"))
                                .lh(d.line_height),
                        ),
                );
            }
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
