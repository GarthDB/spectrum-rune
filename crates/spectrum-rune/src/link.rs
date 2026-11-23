//! Spectrum Link: text that goes somewhere, from `design-data` visuals/link.json (a container around the text, and the
//! keyboard focus ring). Every size, colour and type value comes from [`Tokens`].
//!
//! Built: `primary` (accent) and `secondary` (neutral) variants, `quiet` (underlined only on hover), `standalone`
//! (medium weight; `false` is the inline form: regular weight, to sit in a sentence), hover, down and keyboard focus (colour
//! and the ring), pressing and Enter ([`Link::on_press`]).
//!
//! Limits: the underline is Rune's text underline (its own thickness and offset, not the recipe's 1 pt and 1 pt); a quiet
//! link's underline under keyboard focus is not drawn (Rune's views cannot see focus, a14.31), nor is its colour change
//! under the underline. Not built: `staticColor`.
//!
//! Verification: type, colours and the ring against the oracle (`tests/link.rs`); text widths follow the installed font.

use crate::font;
use crate::interaction::{Interaction, State};
use crate::tokens::{Tokens, Typography};
use rune::{el, rich, text, El, Key, Mods, Span};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Primary,
    Secondary,
}

type Press = Rc<dyn Fn(El) -> El>;

pub struct Link {
    key: String,
    label: String,
    variant: Variant,
    quiet: bool,
    standalone: bool,
    press: Option<Press>,
}

impl Link {
    /// `key` must be unique per instance: the parts are keyed `{key}/container` and `{key}/text`.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Link {
            key: key.into(),
            label: label.into(),
            variant: Variant::default(),
            quiet: false,
            standalone: true,
            press: None,
        }
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    /// Underlined only on hover.
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }
    /// On its own (the default); `false` for a link inside a sentence.
    pub fn standalone(mut self, v: bool) -> Self {
        self.standalone = v;
        self
    }
    /// Called on press and Enter.
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
        let container_key = format!("{}/container", self.key);
        let interaction = Interaction::of(&container_key);
        let state = interaction.state();
        let (rest, hover, down, focus) = match self.variant {
            Variant::Primary => (
                t.accent_content_color_default,
                t.accent_content_color_hover,
                t.accent_content_color_down,
                t.accent_content_color_key_focus,
            ),
            Variant::Secondary => (
                t.neutral_content_color_default,
                t.neutral_content_color_hover,
                t.neutral_content_color_down,
                t.neutral_content_color_key_focus,
            ),
        };
        let ink = match state {
            State::Rest => rest,
            State::Hover => hover,
            State::Down => down,
        };
        let style = font::text_style(&Typography {
            size: t.font_size_100,
            weight: if self.standalone {
                t.medium_font_weight
            } else {
                t.regular_font_weight
            },
            line_height: t.font_size_100 * t.line_height_100,
            ..t.component_m_regular
        });
        // A quiet standalone link is underlined on hover only.
        let underlined = !(self.quiet && self.standalone) || state == State::Hover;
        let label = text(self.label.clone())
            .key(format!("{}/text", self.key))
            .font(style)
            .color(ink)
            .group_color(focus)
            .a11y_hidden();
        let label = if underlined {
            rich(
                self.label.clone(),
                vec![Span::new(0, self.label.len()).underline(ink)],
            )
            .key(format!("{}/text", self.key))
            .font(style)
            .color(ink)
            .group_color(focus)
            .a11y_hidden()
        } else {
            label
        };
        let mut link = el()
            .key(container_key)
            .radius(t.corner_radius_100)
            .role("link")
            .label(self.label)
            .focusable(true)
            .hover_group_focus()
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(label);
        link = interaction.attach(link);
        if let Some(p) = &self.press {
            link = p(link);
        }
        crate::hug(link)
    }
}
