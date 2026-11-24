//! Spectrum Help text: a line of text under a form control, with an alert icon before it for the negative variant, from
//! `design-data` visuals/help-text.json. Every size, gap, colour and type value comes from [`Tokens`].
//!
//! Built: the four sizes (`S` to `XL`), the `neutral` and `negative` variants (the alert icon is the negative variant's, and
//! `hide_icon` removes it), `disabled`.
//!
//! Verification: geometry, type and colours against the oracle (`tests/help_text.rs`); text widths follow the installed
//! font, so only heights, offsets, gaps and colours are compared.

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{icon, row, text, El};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Neutral,
    Negative,
}

pub struct HelpText {
    key: String,
    text: String,
    size: Size,
    variant: Variant,
    hide_icon: bool,
    disabled: bool,
}

impl HelpText {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/icon` and `{key}/text`, as in the
    /// recipe.
    pub fn new(key: impl Into<String>, text: impl Into<String>) -> Self {
        HelpText {
            key: key.into(),
            text: text.into(),
            size: Size::M,
            variant: Variant::Neutral,
            hide_icon: false,
            disabled: false,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    /// Only the negative variant has an icon.
    pub fn hide_icon(mut self, v: bool) -> Self {
        self.hide_icon = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (gap, font_size, glyph) = match self.size {
            Size::S => (t.base_gap_small, t.font_size_75, t.workflow_icon_small),
            Size::M => (t.base_gap_medium, t.font_size_100, t.workflow_icon_medium),
            Size::L => (t.base_gap_large, t.font_size_200, t.workflow_icon_large),
            Size::XL => (
                t.base_gap_extra_large,
                t.font_size_300,
                t.workflow_icon_extra_large,
            ),
        };
        let ink = if self.disabled {
            t.disabled_content_color
        } else if self.variant == Variant::Negative {
            t.negative_content_color_default
        } else {
            t.neutral_subdued_content_color_default
        };
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let part = |name: &str| format!("{}/{name}", self.key);
        // One text line high: a larger icon (the touch scale) overhangs it, as in the oracle.
        let mut help = row()
            .key(part("container"))
            .gap(gap)
            .h(font_size * t.line_height_100);
        if self.variant == Variant::Negative && !self.hide_icon {
            help = help.child(
                icon(&crate::icons::WORKFLOW_ALERT_TRIANGLE, glyph)
                    .key(part("icon"))
                    .color(ink)
                    .a11y_hidden(),
            );
        }
        help.child(text(self.text).key(part("text")).font(style).color(ink))
    }
}
