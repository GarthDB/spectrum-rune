//! Spectrum Field label: the label above or beside a form control, with an asterisk after it when the field is required, from
//! `design-data` visuals/field-label.json. Every size, gap, colour and type value comes from [`Tokens`].
//!
//! Built: the four sizes (`S` to `XL`), the label and the `required` asterisk (`NecessityIndicator::Icon`), `disabled`. Limits,
//! the oracle's: it draws no indicator for `NecessityIndicator::Text` (Spectrum shows "(required)"), and it draws the side
//! label position like the top one (so only the top position is built); static colours are not built.
//!
//! The label is one line high at least `component-height-*` (`min-height`), the text centred in it.
//!
//! Verification: geometry, type and colours against the oracle (`tests/field_label.rs`); text widths follow the installed
//! font, so only heights, offsets, gaps and colours are compared.

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{icon, row, text, El};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NecessityIndicator {
    #[default]
    Icon,
    Text,
}

pub struct FieldLabel {
    key: String,
    label: String,
    size: Size,
    required: bool,
    indicator: NecessityIndicator,
    disabled: bool,
}

impl FieldLabel {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/label` and
    /// `{key}/necessity-indicator`, as in the recipe.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        FieldLabel {
            key: key.into(),
            label: label.into(),
            size: Size::M,
            required: false,
            indicator: NecessityIndicator::Icon,
            disabled: false,
        }
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    pub fn necessity_indicator(mut self, n: NecessityIndicator) -> Self {
        self.indicator = n;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (height, gap, font_size, asterisk) = match self.size {
            Size::S => (
                t.component_height_75,
                t.base_gap_small,
                t.font_size_75,
                &crate::icons::ASTERISK_100,
            ),
            Size::M => (
                t.component_height_100,
                t.base_gap_medium,
                t.font_size_100,
                &crate::icons::ASTERISK_100,
            ),
            Size::L => (
                t.component_height_200,
                t.base_gap_large,
                t.font_size_200,
                &crate::icons::ASTERISK_200,
            ),
            Size::XL => (
                t.component_height_300,
                t.base_gap_extra_large,
                t.font_size_300,
                &crate::icons::ASTERISK_300,
            ),
        };
        let asterisk_size = match self.size {
            Size::S | Size::M => t.asterisk_size_100,
            Size::L => t.asterisk_size_200,
            Size::XL => t.asterisk_size_300,
        };
        let line = font_size * t.line_height_100;
        let ink = if self.disabled {
            t.disabled_content_color
        } else {
            t.neutral_subdued_content_color_default
        };
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: line,
            ..t.component_m_regular
        });
        let part = |name: &str| format!("{}/{name}", self.key);
        let mut label = row()
            .key(part("container"))
            .gap(gap)
            .py(((height - line) / 2.0).max(0.0))
            .child(text(self.label).key(part("label")).font(style).color(ink));
        if self.required && self.indicator == NecessityIndicator::Icon {
            label = label.child(
                icon(asterisk, asterisk_size)
                    .key(part("necessity-indicator"))
                    .color(ink)
                    .a11y_hidden(),
            );
        }
        label
    }
}
