//! Spectrum Radio-group: radio buttons that choose one value, from the foundation's `radio-group` recipe
//! (data 4.2.0, visuals/radio-group.json, `options` slot): a column with no gap (the radio buttons' own
//! height spaces them), or a row of radio buttons `spacing-300` apart when horizontal. Rune's `focus_group` roves focus with the arrow
//! keys and a radio takes its press on arrival, so an arrow moves the selection; `focus_selected` makes
//! Tab land on the selected item.
//!
//! Above the options sits the group's [`FieldLabel`] (the label, with the required asterisk), below them its [`HelpText`]
//! (the description, or in the error state the error message, negative with the alert icon), both as the recipe nests
//! them. [`RadioGroup::hide_label`] keeps the label as the accessible name only. A group without a label or a
//! description draws neither (the oracle draws an empty label and help text's height anyway).
//!
//! Verification: the gap and item heights, and the label, options and help text offsets, against the oracle
//! (`tests/radio_group.rs`); keyboard behaviour in the same test.

use crate::field_label::FieldLabel;
use crate::help_text::{HelpText, Variant};
use crate::radio_button::RadioButton;
use crate::size::Size;
use crate::tokens::Tokens;
use rune::{col, row, El};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

type Change = Rc<dyn Fn(RadioButton, String) -> RadioButton>;

pub struct RadioGroup {
    key: String,
    label: Option<String>,
    items: Vec<(String, String)>,
    value: Option<String>,
    size: Size,
    orientation: Orientation,
    emphasized: bool,
    disabled: bool,
    hide_label: bool,
    required: bool,
    error: bool,
    description: Option<String>,
    change: Option<Change>,
}

impl RadioGroup {
    /// `key` must be unique per instance: items are keyed `{key}/{value}`.
    pub fn new(key: impl Into<String>) -> Self {
        RadioGroup {
            key: key.into(),
            label: None,
            items: Vec::new(),
            value: None,
            size: Size::M,
            orientation: Orientation::Vertical,
            emphasized: false,
            disabled: false,
            hide_label: false,
            required: false,
            error: false,
            description: None,
            change: None,
        }
    }
    /// The group's label, shown above the options (and its accessible name). Always set one; see [`Self::hide_label`].
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Keeps the label as the accessible name only.
    pub fn hide_label(mut self, v: bool) -> Self {
        self.hide_label = v;
        self
    }
    /// An asterisk after the label.
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    /// The text under the options.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// The error state: the description is the error message, negative with the alert icon.
    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }
    /// An option: `value` identifies it (unique in the group), `label` is shown.
    pub fn item(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        let value = value.into();
        debug_assert!(
            self.items.iter().all(|(v, _)| *v != value),
            "RadioGroup `{}`: duplicate item value `{value}` (items are keyed by value)",
            self.key
        );
        self.items.push((value, label.into()));
        self
    }
    /// The selected value; none selected when absent or not an item.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
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
    /// Called with the chosen value on press, Space and arrow-key arrival. Store it as your `value`.
    pub fn on_change<S: 'static>(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        let f = Rc::new(f);
        self.change = Some(Rc::new(move |rb, v| {
            let f = f.clone();
            rb.on_select(move |s: &mut S| f(s, &v))
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        // The recipe binds a gap only for the row: spacing-300 at every size.
        let gap = match self.orientation {
            Orientation::Vertical => 0.0,
            Orientation::Horizontal => t.spacing_300,
        };
        let mut options = match self.orientation {
            Orientation::Vertical => col(),
            Orientation::Horizontal => row(),
        }
        .key(format!("{}/options", self.key))
        .role("radiogroup")
        .focus_group()
        .gap(gap)
        .disabled(self.disabled);
        if let Some(l) = &self.label {
            options = options.label(l.clone());
        }
        if let Some(d) = &self.description {
            options = options.description(d.clone());
        }
        let n = self.items.len();
        for (i, (value, label)) in self.items.iter().enumerate() {
            let selected = self.value.as_deref() == Some(value);
            let mut rb = RadioButton::new(format!("{}/{value}", self.key))
                .label(label.clone())
                .size(self.size)
                .selected(selected)
                .emphasized(self.emphasized)
                .disabled(self.disabled);
            if let Some(c) = &self.change {
                rb = c(rb, value.clone());
            }
            options = options.child(
                rb.build(t)
                    .focus_selected(selected)
                    .position_in_set(i + 1, n),
            );
        }
        let mut root = col().key(self.key.clone());
        if let (Some(l), false) = (&self.label, self.hide_label) {
            root = root.child(
                FieldLabel::new(format!("{}/field-label", self.key), l.clone())
                    .size(self.size)
                    .required(self.required)
                    .disabled(self.disabled)
                    .build(t),
            );
        }
        root = root.child(options);
        if let Some(d) = &self.description {
            root = root.child(
                HelpText::new(format!("{}/help-text", self.key), d.clone())
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
        root
    }
}
