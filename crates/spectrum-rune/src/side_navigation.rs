//! Spectrum Side navigation: a vertical list of links with an optional section title, from `design-data`
//! visuals/side-navigation.json (items, each a row of icon and label with a selection indicator, under a section header).
//! Every size, colour and type value comes from [`Tokens`].
//!
//! Built: the section title, items with an icon, a label and a `selected` (bold, with the 2 pt indicator on the left) or
//! `disabled` look, the list's width (`list-width`, 192 pt; 240 on the touch scale), pressing or Enter on an item
//! ([`SideNavigation::on_select`] with its index) and the arrow keys moving focus along the list. The app keeps which item
//! is selected. Limits, all the oracle's: it draws an item's hover, down and keyboard-focus like rest, so none is drawn beyond
//! Rune's focus ring; it does not indent a nested `level`; and it always draws an icon, so an item without one is
//! unverified (its label then starts at the padding).
//!
//! Verification: geometry, type and colours against the oracle (`tests/side_navigation.rs`); text widths follow the
//! installed font.

use crate::font;
use crate::tokens::{Tokens, Typography};
use rune::{col, icon, row, text, El, Icon, Key, Mods};
use std::rc::Rc;

#[derive(Clone)]
pub struct SideNavigationItem {
    label: String,
    icon: Option<&'static Icon>,
    selected: bool,
    disabled: bool,
}

impl SideNavigationItem {
    pub fn new(label: impl Into<String>) -> Self {
        SideNavigationItem {
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
    /// The current page: bold, with the selection indicator.
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

pub struct SideNavigation {
    key: String,
    title: Option<String>,
    items: Vec<SideNavigationItem>,
    label: String,
    select: Option<Select>,
}

impl SideNavigation {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/section-header`,
    /// `{key}/item-N` (from 1) and `{key}/item-N.icon`, `.label`, `.selection-indicator`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        SideNavigation {
            key: key.into(),
            title: None,
            items: Vec::new(),
            label: "Side navigation".to_owned(),
            select: None,
        }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn item(mut self, item: SideNavigationItem) -> Self {
        self.items.push(item);
        self
    }
    /// The navigation's accessible name (`Side navigation` by default).
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
        let part = |name: &str| format!("{}/{name}", self.key);
        let style = |size: f32, weight: u16| {
            font::text_style(&Typography {
                size,
                weight,
                line_height: size * t.line_height_100,
                ..t.component_m_regular
            })
        };
        let mut list = col()
            .key(part("container"))
            .w(t.list_width)
            .gap(t.list_gap_regular)
            .role("navigation")
            .label(self.label.clone())
            .focus_group();
        if let Some(title) = &self.title {
            list = list.child(
                text(title.clone())
                    .key(part("section-header"))
                    .font(style(t.font_size_75, t.medium_font_weight))
                    .color(t.gray_600),
            );
        }
        for (i, it) in self.items.iter().enumerate() {
            let item_key = part(&format!("item-{}", i + 1));
            let (ink, weight) = if it.disabled {
                (t.disabled_content_color, t.regular_font_weight)
            } else if it.selected {
                (t.neutral_content_color_default, t.bold_font_weight)
            } else {
                (
                    t.neutral_subdued_content_color_default,
                    t.regular_font_weight,
                )
            };
            let mut line = row()
                .key(item_key.clone())
                .w_full()
                .min_h(t.base_height_medium)
                .pl(t.base_padding_horizontal_medium)
                .pr(t.base_padding_horizontal_medium)
                .gap(t.base_gap_medium)
                .center_y()
                .role("link")
                .label(it.label.clone())
                .disabled(it.disabled)
                .focusable(!it.disabled);
            if it.selected {
                line = line.child(el_indicator(
                    t,
                    format!("{item_key}.selection-indicator"),
                    ink,
                ));
            }
            if let Some(ic) = it.icon {
                line = line.child(
                    icon(ic, t.workflow_icon_medium)
                        .key(format!("{item_key}.icon"))
                        .color(ink)
                        .a11y_hidden(),
                );
            }
            line = line.child(
                text(it.label.clone())
                    .key(format!("{item_key}.label"))
                    .font(style(t.font_size_100, weight))
                    .color(ink)
                    .a11y_hidden(),
            );
            if !it.disabled {
                if let Some(s) = &self.select {
                    line = s(line, i);
                }
            }
            list = list.child(line);
        }
        crate::hug(list)
    }
}

/// The 2 pt bar at the item's left edge, as tall as a medium visual and centred on the row.
fn el_indicator(t: &Tokens, key: String, fill: rune::Color) -> El {
    rune::el()
        .key(key)
        .absolute(
            0.0,
            (t.base_height_medium - t.base_visual_size_medium) / 2.0,
        )
        .w(t.bar_thickness_extra_small)
        .h(t.base_visual_size_medium)
        .fill(fill)
}
