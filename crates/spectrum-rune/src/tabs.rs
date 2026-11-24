//! Spectrum Tabs: a row of tab items, each an icon and a label, with a selection indicator under the selected one, from
//! `design-data` visuals/tabs.json. Every size, gap, colour and type value comes from [`Tokens`].
//!
//! Built: horizontal tabs with an icon and a label, the four sizes (`S` to `XL`), the two densities (`regular` and `compact`:
//! shorter tabs), the `selected` item (the content colour and the 2 pt indicator under it), `disabled` items, pressing or
//! Enter on a tab ([`Tabs::on_select`] with its index) and the arrow keys moving focus along the row (Rune activates a tab as
//! focus arrives on it, so an arrow key also calls `on_select`). The app keeps which tab
//! is selected and shows its panel. Limits, mostly the oracle's: it draws a tab's hover, down and keyboard-focus like rest (so
//! none beyond Rune's focus ring) and gives the vertical orientation's selected tab no indicator (so vertical tabs are not
//! built); the selected tab is announced as checked (Rune has no `aria-selected`); it always draws an icon, so a tab without one is unverified (its label then starts at the tab's edge).
//!
//! Verification: geometry, type and colours against the oracle (`tests/tabs.rs`); text widths follow the installed font, so
//! the tabs' widths are not compared, only the gaps between them.

use crate::font;
use crate::tokens::{Tokens, Typography};
use crate::Size;
use rune::{col, el, icon, row, text, El, Icon, Key, Mods};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Regular,
    /// Shorter tabs.
    Compact,
}

#[derive(Clone)]
pub struct TabItem {
    label: String,
    icon: Option<&'static Icon>,
    selected: bool,
    disabled: bool,
}

impl TabItem {
    pub fn new(label: impl Into<String>) -> Self {
        TabItem {
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

pub struct Tabs {
    key: String,
    items: Vec<TabItem>,
    size: Size,
    density: Density,
    label: String,
    select: Option<Select>,
}

impl Tabs {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/tab-item-N` (from 1) and
    /// `{key}/tab-item-N.icon`, `.label`, `.selection-indicator`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        Tabs {
            key: key.into(),
            items: Vec::new(),
            size: Size::M,
            density: Density::Regular,
            label: "Tabs".to_owned(),
            select: None,
        }
    }
    pub fn item(mut self, item: TabItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn density(mut self, d: Density) -> Self {
        self.density = d;
        self
    }
    /// The tab list's accessible name (`Tabs` by default).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Called with the tab's index on press and on Enter. A disabled tab does not call it.
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
        let compact = self.density == Density::Compact;
        let (spacing, height, gap, glyph, font_size) = match self.size {
            Size::S => (
                t.spacing_350,
                if compact {
                    t.component_height_75
                } else {
                    t.base_height_large
                },
                t.base_gap_small,
                t.workflow_icon_small,
                t.font_size_75,
            ),
            Size::M => (
                if compact {
                    t.tab_item_to_tab_item_compact_horizontal_medium
                } else {
                    t.spacing_400
                },
                if compact {
                    t.component_height_100
                } else {
                    t.base_height_extra_large
                },
                t.base_gap_medium,
                t.workflow_icon_medium,
                t.font_size_100,
            ),
            Size::L => (
                t.spacing_500,
                if compact {
                    t.component_height_200
                } else {
                    t.base_height_2x_large
                },
                t.base_gap_large,
                t.workflow_icon_large,
                t.font_size_200,
            ),
            Size::XL => (
                t.spacing_600,
                if compact {
                    t.component_height_300
                } else {
                    t.base_height_3x_large
                },
                t.base_gap_extra_large,
                t.workflow_icon_extra_large,
                t.font_size_300,
            ),
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let style = font::text_style(&Typography {
            size: font_size,
            weight: t.regular_font_weight,
            line_height: font_size * t.line_height_100,
            ..t.component_m_regular
        });
        let mut list = row()
            .key(part("container"))
            .gap(spacing)
            .role("tablist")
            .label(self.label.clone())
            .focus_group();
        for (i, it) in self.items.iter().enumerate() {
            let item_key = part(&format!("tab-item-{}", i + 1));
            let ink = if it.disabled {
                t.disabled_content_color
            } else if it.selected {
                t.neutral_content_color_default
            } else {
                t.neutral_subdued_content_color_default
            };
            let mut tab = row()
                .key(item_key.clone())
                .min_h(height)
                .gap(gap)
                .center_y()
                .role("tab")
                .label(it.label.clone())
                .checked(it.selected)
                .disabled(it.disabled)
                .focusable(!it.disabled);
            if let Some(ic) = it.icon {
                tab = tab.child(
                    icon(ic, glyph)
                        .key(format!("{item_key}.icon"))
                        .color(ink)
                        .a11y_hidden(),
                );
            }
            tab = tab.child(
                text(it.label.clone())
                    .key(format!("{item_key}.label"))
                    .font(style)
                    .color(ink)
                    .a11y_hidden(),
            );
            if it.selected {
                tab = tab.child(
                    el().key(format!("{item_key}.selection-indicator"))
                        .absolute(0.0, height - t.bar_thickness_extra_small)
                        .w_full()
                        .h(t.bar_thickness_extra_small)
                        .fill(t.neutral_content_color_default),
                );
            }
            if !it.disabled {
                if let Some(s) = &self.select {
                    tab = s(tab, i);
                }
            }
            list = list.child(tab);
        }
        crate::hug(col().child(list))
    }
}
