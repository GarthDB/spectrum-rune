//! Spectrum Radio-group: radio buttons that choose one value, from the `radio-group` recipe in
//! `platforms/rune/extensions/visuals/radio-group.json` (the foundation has none yet): a column, or a row
//! when horizontal, of radio buttons `group-gap` apart. Rune's `focus_group` roves focus with the arrow
//! keys and a radio takes its press on arrival, so an arrow moves the selection; `focus_selected` makes
//! Tab land on the selected item.
//!
//! Verification: the gap and item heights against the oracle (`tests/radio_group.rs`); keyboard
//! behaviour in the same test. Not drawn: the group label (the recipe has none; it is the accessible
//! name only), description, required and error message.

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
            change: None,
        }
    }
    /// The group's accessible name. Always set one.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
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
        let gap = match self.size {
            Size::S => t.group_gap_small,
            Size::M => t.group_gap_medium,
            Size::L => t.group_gap_large,
            Size::XL => t.group_gap_extra_large,
        };
        let mut root = match self.orientation {
            Orientation::Vertical => col(),
            Orientation::Horizontal => row(),
        }
        .key(self.key.clone())
        .role("radiogroup")
        .focus_group()
        .gap(gap)
        .disabled(self.disabled);
        if let Some(l) = &self.label {
            root = root.label(l.clone());
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
            root = root.child(
                rb.build(t)
                    .focus_selected(selected)
                    .position_in_set(i + 1, n),
            );
        }
        root
    }
}
