//! Spectrum Tag: a small label on a pill, from `design-data` visuals/tag.json (a container with a label, the keyboard
//! focus ring and the parts below). Every size, colour, radius and type value comes from [`Tokens`].
//!
//! Built: the label, the three sizes (`S`, `M`, `L`), `selected` (a neutral fill, or the accent one with `emphasized`),
//! `disabled`, and, when [`Tag::on_press`] makes it interactive, hover, down and keyboard focus (the hover fill and ring).
//! A tag with no handler is static: no states. [`Tag::removable`] adds the recipe's remove button (a [`ClearButton`]); its
//! press, and Delete or Backspace on a focused tag, call [`Tag::on_remove`]. The recipe draws that button's cross in the
//! neutral content colour even on a selected tag, so on the neutral selected fill it is dark on dark: we follow the oracle.
//! Not built (the recipe has them): `hasAvatar` (needs the Avatar) and `icon`; `isError` is not visual in the recipe.
//!
//! Verification: geometry, fills and colours against the oracle (`tests/tag.rs`), the focus fill and ring from pixels
//! (`tests/tag_pixels.rs`).

use crate::clear_button::ClearButton;
use crate::font;
use crate::interaction::{Interaction, State};
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{row, text, El, Key, Mods};
use std::rc::Rc;

type Press = Rc<dyn Fn(El) -> El>;
type Remove = Rc<dyn Fn(ClearButton) -> ClearButton>;

pub struct Tag {
    key: String,
    label: String,
    size: Size,
    selected: bool,
    emphasized: bool,
    disabled: bool,
    press: Option<Press>,
    removable: bool,
    remove: Option<Remove>,
    remove_key: Option<Press>,
}

impl Tag {
    /// `key` must be unique per instance: the parts are keyed `{key}/container` and `{key}/label`.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Tag {
            key: key.into(),
            label: label.into(),
            size: Size::M,
            selected: false,
            emphasized: false,
            disabled: false,
            press: None,
            removable: false,
            remove: None,
            remove_key: None,
        }
    }
    /// `S`, `M` or `L`; the recipe has no `XL` (it draws as `L`).
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    /// With `selected`, the accent fill instead of the neutral one.
    pub fn emphasized(mut self, v: bool) -> Self {
        self.emphasized = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Shows the remove button (the recipe's `isRemovable`); give it a handler with [`Tag::on_remove`].
    pub fn removable(mut self, v: bool) -> Self {
        self.removable = v;
        self
    }
    /// Called when the remove button is pressed, and on Delete or Backspace while the tag has focus. Implies `removable`.
    pub fn on_remove<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        let g = f.clone();
        self.removable = true;
        self.remove = Some(Rc::new(move |b: ClearButton| {
            let f = f.clone();
            b.on_press(move |s: &mut S| f(s))
        }));
        self.remove_key = Some(Rc::new(move |e: El| {
            let g = g.clone();
            e.on_key(move |s: &mut S, ev, tx| {
                let hit =
                    matches!(ev.key, Key::Backspace | Key::Delete) && ev.mods == Mods::default();
                if hit && !tx.key_repeat() {
                    g(s);
                }
                hit
            })
        }));
        self
    }
    /// Makes the tag interactive: called on press and on Enter or Space. A disabled tag does not call it.
    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        self.press = Some(Rc::new(move |e: El| {
            let (a, b) = (f.clone(), f.clone());
            e.on_press(move |s: &mut S| a(s))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit = matches!(ev.key, Key::Enter) && ev.mods == Mods::default();
                    if hit && !tx.key_repeat() {
                        b(s);
                    }
                    hit
                })
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (height, pad, gap, radius, font_size, min_width) = match self.size {
            Size::S => (
                t.base_height_small,
                t.base_padding_horizontal_small,
                t.base_gap_small,
                t.corner_radius_medium_size_small,
                t.font_size_75,
                t.tag_minimum_width_small,
            ),
            Size::M => (
                t.base_height_medium,
                t.base_padding_horizontal_medium,
                t.base_gap_medium,
                t.corner_radius_medium_size_medium,
                t.font_size_100,
                t.tag_minimum_width_medium,
            ),
            Size::L | Size::XL => (
                t.base_height_large,
                t.base_padding_horizontal_large,
                t.base_gap_large,
                t.corner_radius_medium_size_large,
                t.font_size_200,
                t.tag_minimum_width_large,
            ),
        };
        // A removable tag's right padding is the small gap, and the label is a full padding from the button.
        let (pad_right, gap) = if self.removable {
            (
                match self.size {
                    Size::S => t.base_gap_small,
                    Size::M => t.base_gap_medium,
                    Size::L | Size::XL => t.base_gap_large,
                },
                pad,
            )
        } else {
            (pad, gap)
        };
        let button_size = self.size;
        let container_key = format!("{}/container", self.key);
        let interactive = (self.press.is_some() || self.remove.is_some()) && !self.disabled;
        let interaction = Interaction::of(&container_key);
        let state = if interactive {
            interaction.state()
        } else {
            State::Rest
        };
        let (fill, ink, focus_fill, focus_ink) = if self.disabled {
            (
                t.disabled_background_color,
                t.disabled_content_color,
                t.disabled_background_color,
                t.disabled_content_color,
            )
        } else if self.selected && self.emphasized {
            let white = t.static_white_text_color;
            (
                match state {
                    State::Rest => t.accent_background_color_default,
                    State::Hover => t.accent_background_color_hover,
                    State::Down => t.accent_background_color_down,
                },
                white,
                t.accent_background_color_hover,
                white,
            )
        } else if self.selected {
            (
                match state {
                    State::Rest => t.neutral_background_color_selected_default,
                    State::Hover => t.neutral_background_color_selected_hover,
                    State::Down => t.neutral_background_color_selected_down,
                },
                t.gray_25,
                t.neutral_background_color_selected_hover,
                t.gray_25,
            )
        } else {
            (
                match state {
                    State::Hover => t.gray_200,
                    State::Rest | State::Down => t.gray_100,
                },
                match state {
                    State::Rest => t.neutral_content_color_default,
                    State::Hover => t.neutral_content_color_hover,
                    State::Down => t.neutral_content_color_down,
                },
                t.gray_200,
                t.neutral_content_color_key_focus,
            )
        };
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.medium_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let mut tag = row()
            .key(container_key)
            .min_w(min_width)
            .min_h(height)
            .pl(pad)
            .pr(pad_right)
            .gap(gap)
            .center_y()
            .radius(radius)
            .fill(fill)
            .role(if self.press.is_some() {
                "button"
            } else {
                "text"
            })
            .label(self.label.clone())
            .disabled(self.disabled)
            .child(
                text(self.label)
                    .key(format!("{}/label", self.key))
                    .font(style)
                    .color(ink)
                    .group_color(focus_ink)
                    .a11y_hidden(),
            );
        if self.removable {
            let mut b = ClearButton::new(format!("{}/clear-button", self.key))
                .size(button_size)
                .label("Remove")
                .disabled(self.disabled);
            if let Some(r) = &self.remove {
                b = r(b);
            }
            tag = tag.child(b.build(t));
        }
        if interactive {
            tag = tag
                .focusable(true)
                .hover_group_focus()
                .focus_fill(focus_fill)
                .focus_ring(
                    t.focus_indicator_thickness,
                    t.focus_ring_gap,
                    t.focus_indicator_color,
                );
            tag = interaction.attach(tag);
            if let Some(p) = &self.press {
                tag = p(tag);
            }
            if let Some(k) = &self.remove_key {
                tag = k(tag);
            }
        }
        crate::hug(tag)
    }
}
