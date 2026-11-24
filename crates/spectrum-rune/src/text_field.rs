//! Spectrum Text field: a field label, a bordered single-line field holding the text (and a checkmark when it is valid) and
//! help text, from `design-data` visuals/text-field.json. Every size, colour and type value comes from [`Tokens`]; the label
//! is a [`FieldLabel`] and the help text a [`HelpText`].
//!
//! The text is Rune's own editor: the app keeps a [`TextEdit`] and binds it with [`TextField::on_edit`]
//! (`.on_edit(|s: &mut App| &mut s.name)`), so selection, caret, clipboard, IME and undo are Rune's. Enter calls
//! [`TextField::on_submit`].
//!
//! Built: the four sizes (`S` to `XL`), the label (with the required asterisk) and help text, `error` (the negative border and
//! help text), the `valid` checkmark (shown when not in error), `disabled`, hover (the stronger border and the hover text
//! colour) and keyboard focus (the focus ring around the field, the focus text colour; Rune also draws it on pointer focus,
//! as a text field shows). Limits, mostly the oracle's: the field's width is the size's `field-width-*` token (the oracle
//! ignores a `width` option); a field without a label draws none (the oracle reserves an empty label's height); no
//! character count, side label position or placeholder (the recipe has none).
//!
//! Verification: geometry, type and colours against the oracle (`tests/text_field.rs`), the border and the focus ring from
//! pixels; text widths follow the installed font, so only heights, offsets and colours are compared.

use crate::field_label::FieldLabel;
use crate::font;
use crate::help_text::{HelpText, Variant};
use crate::icons::CHECKMARK_100;
use crate::interaction::{Interaction, State};
use crate::tokens::Tokens;
use crate::Size;
use rune::{col, icon, row, text_input, El, TextEdit, Tx};
use std::rc::Rc;

type Bind = Rc<dyn Fn(El) -> El>;

pub struct TextField {
    key: String,
    edit: TextEdit,
    label: Option<String>,
    help: Option<String>,
    size: Size,
    required: bool,
    error: bool,
    valid: bool,
    disabled: bool,
    /// The text area: a multi-line editor, a minimum height, vertical padding, the checkmark at the top.
    pub(crate) area: Option<AreaOptions>,
    binds: Vec<Bind>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct AreaOptions {
    pub(crate) min_height: Option<f32>,
}

impl TextField {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `/field`, `/value`, `/validation-marker`,
    /// `/field-label/..` and `/help-text/..`, as in the recipe.
    pub fn new(key: impl Into<String>, edit: &TextEdit) -> Self {
        TextField {
            key: key.into(),
            edit: edit.clone(),
            label: None,
            help: None,
            size: Size::M,
            required: false,
            error: false,
            valid: false,
            disabled: false,
            area: None,
            binds: Vec::new(),
        }
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// The help text; in the error state it is the error message, in the negative colour with the alert icon.
    pub fn help_text(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// An asterisk after the label.
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }
    /// The checkmark at the field's end (not shown in the error state).
    pub fn valid(mut self, v: bool) -> Self {
        self.valid = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Routes edits to the app's [`TextEdit`].
    pub fn on_edit<S: 'static>(
        mut self,
        f: impl for<'a> Fn(&'a mut S) -> &'a mut TextEdit + Clone + 'static,
    ) -> Self {
        self.binds.push(Rc::new(move |e: El| e.on_edit(f.clone())));
        self
    }
    /// Called on Enter.
    pub fn on_submit<S: 'static>(mut self, f: impl Fn(&mut S, &mut Tx) + Clone + 'static) -> Self {
        self.binds
            .push(Rc::new(move |e: El| e.on_submit(f.clone())));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (width, height, pad, radius, pad_y) = match self.size {
            Size::S => (
                t.field_width_small,
                t.base_height_small,
                t.base_padding_horizontal_small,
                t.corner_radius_medium_size_small,
                t.base_padding_vertical_small,
            ),
            Size::M => (
                t.field_width_medium,
                t.base_height_medium,
                t.base_padding_horizontal_medium,
                t.corner_radius_medium_size_medium,
                t.base_padding_vertical_medium,
            ),
            Size::L => (
                t.field_width_large,
                t.base_height_large,
                t.base_padding_horizontal_large,
                t.corner_radius_medium_size_large,
                t.base_padding_vertical_large,
            ),
            Size::XL => (
                t.field_width_extra_large,
                t.base_height_extra_large,
                t.base_padding_horizontal_extra_large,
                t.corner_radius_medium_size_extra_large,
                t.base_padding_vertical_extra_large,
            ),
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let field_key = part("field");
        let interaction = Interaction::of(&field_key);
        let hovered = !self.disabled && interaction.state() != State::Rest;
        let stroke = if self.disabled {
            t.disabled_border_color
        } else if self.error {
            t.negative_border_color_default
        } else if hovered {
            t.gray_400
        } else {
            t.gray_300
        };
        let (ink, ink_hover) = if self.disabled {
            (t.disabled_content_color, t.disabled_content_color)
        } else if hovered {
            (t.neutral_content_color_hover, t.neutral_content_color_hover)
        } else {
            (
                t.neutral_content_color_default,
                t.neutral_content_color_hover,
            )
        };
        let style = font::text_style(&match self.size {
            Size::S => t.component_s_regular,
            Size::M => t.component_m_regular,
            Size::L => t.component_l_regular,
            Size::XL => t.component_xl_regular,
        });
        let mut value = text_input(&self.edit, style).key(part("value"));
        if self.area.is_none() {
            value = value.single_line();
        }
        let mut value = value
            .grow(1.0)
            .color(ink)
            .group_color(ink_hover)
            .caret(
                t.neutral_content_color_default,
                t.accent_background_color_default.alpha(0.3),
            )
            .disabled(self.disabled);
        if let Some(l) = &self.label {
            value = value.label(l.clone());
        }
        if !self.disabled {
            for b in &self.binds {
                value = b(value);
            }
        }
        let mut field = row()
            .key(field_key)
            .px(pad)
            .radius(radius)
            .fill(t.gray_25)
            .border(t.border_width_200, stroke)
            .hover_group_focus()
            .focus_within_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .child(value);
        field = match self.area {
            Some(a) => field
                .min_h(a.min_height.unwrap_or(height))
                .py(pad_y)
                .items_start(),
            None => field.h(height).center_y(),
        };
        if self.valid && !self.error && !self.disabled {
            field = field.child(
                icon(&CHECKMARK_100, 16.0)
                    .key(part("validation-marker"))
                    .color(t.positive_visual_color)
                    .a11y_hidden(),
            );
        }
        if !self.disabled {
            field = interaction.attach_hover(field);
        }
        let mut c = col().key(part("container")).w(width);
        if let Some(l) = &self.label {
            c = c.child(
                FieldLabel::new(part("field-label"), l.clone())
                    .size(self.size)
                    .required(self.required)
                    .disabled(self.disabled)
                    .build(t),
            );
        }
        c = c.child(field);
        if let Some(h) = &self.help {
            c = c.child(
                HelpText::new(part("help-text"), h.clone())
                    .size(self.size)
                    .variant(if self.error {
                        Variant::Negative
                    } else {
                        Variant::Neutral
                    })
                    .disabled(self.disabled)
                    .build(t),
            );
        }
        c
    }
}

/// Spectrum Text area: the [`TextField`] as a multi-line editor, from `design-data` visuals/text-area.json. The field is at
/// least the size's height and grows with the text; its text wraps, with `base-padding-vertical-*` above and below, and the
/// valid checkmark sits at its top. Everything else (label, help text, states, the editor) is the text field's.
///
/// Limits, the oracle's: it draws a single line, truncated, and ignores the `height` option, so only the empty or one-line
/// field is verified; the wrapped, growing field is Rune's layout, unverified. No drag handle (`hideDragIcon`), character
/// count, input types or placeholder.
pub struct TextArea(TextField);

impl TextArea {
    /// `key` must be unique per instance; the parts are the text field's (`{key}/field`, `{key}/value`, ...).
    pub fn new(key: impl Into<String>, edit: &TextEdit) -> Self {
        let mut f = TextField::new(key, edit);
        f.area = Some(AreaOptions::default());
        TextArea(f)
    }
    pub fn label(self, l: impl Into<String>) -> Self {
        TextArea(self.0.label(l))
    }
    pub fn help_text(self, h: impl Into<String>) -> Self {
        TextArea(self.0.help_text(h))
    }
    pub fn size(self, s: Size) -> Self {
        TextArea(self.0.size(s))
    }
    pub fn required(self, v: bool) -> Self {
        TextArea(self.0.required(v))
    }
    pub fn error(self, v: bool) -> Self {
        TextArea(self.0.error(v))
    }
    pub fn valid(self, v: bool) -> Self {
        TextArea(self.0.valid(v))
    }
    pub fn disabled(self, v: bool) -> Self {
        TextArea(self.0.disabled(v))
    }
    /// The field's minimum height, instead of the size's (the oracle ignores it).
    pub fn height(mut self, h: f32) -> Self {
        self.0.area = Some(AreaOptions {
            min_height: Some(h),
        });
        self
    }
    /// Routes edits to the app's [`TextEdit`]; Enter then breaks the line (Rune's plain Return submits unless told otherwise).
    pub fn on_edit<S: 'static>(
        mut self,
        f: impl for<'a> Fn(&'a mut S) -> &'a mut TextEdit + Clone + 'static,
    ) -> Self {
        let g = f.clone();
        self.0 = self.0.on_edit(f);
        self.0.binds.push(Rc::new(move |e: El| {
            let g = g.clone();
            e.on_submit(move |s: &mut S, _| g(s).insert("\n"))
        }));
        self
    }
    pub fn build(self, t: &Tokens) -> El {
        self.0.build(t)
    }
}
