//! Spectrum Segmented control: a rounded track holding a row of items, each an icon and a label, the selected one on a raised
//! fill, from `design-data` visuals/segmented-control.json. Every size, gap, colour and type value comes from [`Tokens`].
//!
//! Built: horizontal segmented controls with an icon and a label, the four sizes (`S` to `XL`), the `selected` item, `disabled`
//! items, pressing or Enter on an item ([`SegmentedControl::on_select`] with its index) and the arrow keys moving focus along
//! the row (Rune activates an item as focus arrives on it, so an arrow key also calls `on_select`). The app keeps which item
//! is selected. Limits, mostly the oracle's: it draws every item's label in the subdued colour, the selected one too (the
//! raised fill alone marks it), and hover, down and keyboard-focus like rest (so none beyond Rune's focus ring); vertical
//! orientation and `hideTracker` are not built; it always draws an icon, so an item without one is unverified. The selected
//! item is announced as checked (Rune has no `aria-selected`).
//!
//! The item's height is the label's line plus the vertical padding, as in the oracle: a larger icon (mobile) overhangs it
//! rather than stretching the track.
//!
//! Verification: geometry, type and colours against the oracle (`tests/segmented_control.rs`); text widths follow the
//! installed font, so the items' widths are not compared, only the gaps between them.

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{col, icon, row, text, El, Icon, Key, Mods};
use std::rc::Rc;

#[derive(Clone)]
pub struct SegmentedItem {
    label: String,
    icon: Option<&'static Icon>,
    selected: bool,
    disabled: bool,
}

impl SegmentedItem {
    pub fn new(label: impl Into<String>) -> Self {
        SegmentedItem {
            label: label.into(),
            icon: None,
            selected: false,
            disabled: false,
        }
    }
    /// A workflow icon (`spectrum_rune::icons`).
    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

type Select = Rc<dyn Fn(El, usize) -> El>;

pub struct SegmentedControl {
    key: String,
    items: Vec<SegmentedItem>,
    size: Size,
    label: String,
    select: Option<Select>,
}

impl SegmentedControl {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/item-N` (from 1) and
    /// `{key}/item-N.icon`, `.label`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        SegmentedControl {
            key: key.into(),
            items: Vec::new(),
            size: Size::M,
            label: "Segmented control".to_owned(),
            select: None,
        }
    }
    pub fn item(mut self, item: SegmentedItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// The control's accessible name (`Segmented control` by default).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Called with the item's index on press and on Enter. A disabled item does not call it.
    pub fn on_select<S: 'static>(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        let f = Rc::new(f);
        self.select = Some(Rc::new(move |e: El, i: usize| {
            let (a, b) = (f.clone(), f.clone());
            e.on_press(move |s: &mut S| a(s, i))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit = matches!(ev.key, Key::Enter) && ev.mods == Mods::default();
                    if hit && !tx.key_repeat() {
                        b(s, i);
                    }
                    hit
                })
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let (pad_x, pad_y, gap, glyph, font_size, radius) = match self.size {
            Size::S => (
                t.base_padding_horizontal_small,
                t.base_padding_vertical_small,
                t.base_gap_small,
                t.workflow_icon_small,
                t.font_size_75,
                t.corner_radius_500,
            ),
            Size::M => (
                t.base_padding_horizontal_medium,
                t.base_padding_vertical_medium,
                t.base_gap_medium,
                t.workflow_icon_medium,
                t.font_size_100,
                t.corner_radius_m,
            ),
            Size::L => (
                t.base_padding_horizontal_large,
                t.base_padding_vertical_large,
                t.base_gap_large,
                t.workflow_icon_large,
                t.font_size_200,
                t.corner_radius_500,
            ),
            Size::XL => (
                t.base_padding_horizontal_extra_large,
                t.base_padding_vertical_extra_large,
                t.base_gap_extra_large,
                t.workflow_icon_extra_large,
                t.font_size_300,
                t.corner_radius_500,
            ),
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let mut track = row()
            .key(part("container"))
            .gap(t.spacing_75)
            .radius(radius)
            .fill(t.gray_100)
            .role("radiogroup")
            .label(self.label.clone())
            .focus_group();
        for (i, it) in self.items.iter().enumerate() {
            let item_key = part(&format!("item-{}", i + 1));
            let ink = if it.disabled {
                t.disabled_content_color
            } else {
                t.neutral_subdued_content_color_default
            };
            let mut item = row()
                .key(item_key.clone())
                .px(pad_x)
                .h(2.0 * pad_y + font_size * t.line_height_100)
                .gap(gap)
                .center_y()
                .radius(radius)
                .role("radio")
                .label(it.label.clone())
                .checked(it.selected)
                .disabled(it.disabled)
                .focusable(!it.disabled);
            if it.selected {
                item = item.fill(t.gray_25);
            }
            if let Some(ic) = it.icon {
                item = item.child(
                    icon(ic, glyph)
                        .key(format!("{item_key}.icon"))
                        .color(ink)
                        .a11y_hidden(),
                );
            }
            item = item.child(
                text(it.label.clone())
                    .key(format!("{item_key}.label"))
                    .font(style)
                    .color(ink)
                    .a11y_hidden(),
            );
            if !it.disabled {
                if let Some(s) = &self.select {
                    item = s(item, i);
                }
            }
            track = track.child(item);
        }
        crate::hug(col().child(track))
    }
}
