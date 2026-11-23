//! Spectrum Clear button: a round button with a cross, for clearing a field or removing an item, from `design-data`
//! visuals/clear-button.json (a circular container with a cross icon, and the keyboard focus ring). Every size and colour
//! comes from [`Tokens`].
//!
//! Built: the four sizes, `disabled`, hover and down (the cross takes the hover colour), keyboard focus (the ring) and
//! [`ClearButton::on_press`]. Not built: `staticColor`. The recipe gives the button no fill in any state.
//!
//! Verification: geometry and colours against the oracle (`tests/clear_button.rs`), the ring from pixels
//! (`tests/clear_button_pixels.rs`).

use crate::icons::{CROSS_100, CROSS_200, CROSS_300, CROSS_75};
use crate::interaction::{Interaction, State};
use crate::tokens::Tokens;
use crate::Size;
use rune::{el, icon, El, Key, Mods};
use std::rc::Rc;

type Press = Rc<dyn Fn(El) -> El>;

pub struct ClearButton {
    key: String,
    label: String,
    size: Size,
    disabled: bool,
    press: Option<Press>,
}

impl ClearButton {
    /// `key` must be unique per instance: the parts are keyed `{key}/container` and `{key}/icon`.
    pub fn new(key: impl Into<String>) -> Self {
        ClearButton {
            key: key.into(),
            label: "Clear".to_owned(),
            size: Size::M,
            disabled: false,
            press: None,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// The accessible name (`Clear` by default): the button has no visible label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Called on press, Space and Enter.
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
        let (side, glyph, glyph_size) = match self.size {
            Size::S => (t.base_height_small, &CROSS_75, t.cross_size_75),
            Size::M => (t.base_height_medium, &CROSS_100, t.cross_size_100),
            Size::L => (t.base_height_large, &CROSS_200, t.cross_size_200),
            Size::XL => (t.base_height_extra_large, &CROSS_300, t.cross_size_300),
        };
        let container_key = format!("{}/container", self.key);
        let interaction = Interaction::of(&container_key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let ink = if self.disabled {
            t.disabled_content_color
        } else {
            match state {
                State::Rest => t.neutral_content_color_default,
                State::Hover => t.neutral_content_color_hover,
                State::Down => t.neutral_content_color_down,
            }
        };
        let mut button = el()
            .key(container_key)
            .w(side)
            .h(side)
            .radius(side / 2.0)
            .center()
            .role("button")
            .label(self.label)
            .disabled(self.disabled)
            .child(
                icon(glyph, glyph_size)
                    .key(format!("{}/icon", self.key))
                    .color(ink)
                    .group_color(t.neutral_content_color_key_focus)
                    .a11y_hidden(),
            );
        if !self.disabled {
            button = button.focusable(true).hover_group_focus().focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            );
            button = interaction.attach(button);
            if let Some(p) = &self.press {
                button = p(button);
            }
        }
        button
    }
}
