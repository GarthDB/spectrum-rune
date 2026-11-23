//! Spectrum Menu: a column of selectable items, from `design-data` visuals/menu.json (the recipe's menu-items
//! and their label, optional value and selection checkmark). The container is `notVisual` in the recipe: a
//! menu has no surface of its own here, put it in a popover or on a layer. Every size, colour, radius and
//! type value comes from [`Tokens`].
//!
//! Built: `Selection::None` (plain items) and `Selection::Single` (a checkmark on the selected item, the others
//! indented to line up with its label), a trailing `value` text, `disabled` items, hover, down and keyboard
//! focus. Not built (the recipe has them): multiple selection with a checkbox or switch, an item icon or
//! thumbnail, a section header and collapsing. In the oracle a state applies to every item; here it is the item
//! under the pointer or the keyboard.
//!
//! Keyboard: Rune's `focus_group` moves focus with the arrow keys; Enter and Space choose an item, Escape
//! calls [`Menu::on_dismiss`] (the catalog's intents: `navigate-options`, `activate`, `dismiss`).
//!
//! Verification: geometry and colours against the oracle (`tests/menu.rs`).

use crate::font;
use crate::icons::CHECKMARK_100;
use crate::interaction::{Interaction, State};
use crate::tokens::{Tokens, Typography};
use rune::{col, el, icon, row, text, El, Key, Mods};
use std::rc::Rc;

pub use crate::size::Size;

/// How the menu marks what is chosen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    /// No selection: plain items.
    #[default]
    None,
    /// One item is marked with a checkmark.
    Single,
}

#[derive(Clone, Debug)]
pub struct MenuItem {
    id: String,
    label: String,
    value: Option<String>,
    selected: bool,
    disabled: bool,
}

impl MenuItem {
    /// `id` identifies the item (unique in the menu) and keys it; `label` is shown.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        MenuItem {
            id: id.into(),
            label: label.into(),
            value: None,
            selected: false,
            disabled: false,
        }
    }
    /// Trailing text, such as a shortcut.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
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

struct Dims {
    min_height: f32,
    pad: f32,
    pad_v: f32,
    gap: f32,
    radius: f32,
    /// Where an unselected item's label starts when the menu shows selection.
    not_selected: f32,
    check: f32,
    label: Typography,
    value: Typography,
}

impl Size {
    fn menu_dims(self, t: &Tokens) -> Dims {
        let (min_height, pad, pad_v, gap, radius, not_selected, check) = match self {
            Size::S => (
                t.base_height_small,
                t.base_padding_horizontal_small,
                t.base_padding_vertical_small,
                t.base_gap_small,
                t.corner_radius_400,
                t.menu_item_edge_to_content_not_selected_small,
                t.ui_icon_small,
            ),
            Size::M => (
                t.base_height_medium,
                t.base_padding_horizontal_medium,
                t.base_padding_vertical_medium,
                t.base_gap_medium,
                t.corner_radius_500,
                t.menu_item_edge_to_content_not_selected_medium,
                t.ui_icon_medium,
            ),
            Size::L => (
                t.base_height_large,
                t.base_padding_horizontal_large,
                t.base_padding_vertical_large,
                t.base_gap_large,
                t.corner_radius_600,
                t.menu_item_edge_to_content_not_selected_large,
                t.ui_icon_large,
            ),
            Size::XL => (
                t.base_height_extra_large,
                t.base_padding_horizontal_extra_large,
                t.base_padding_vertical_extra_large,
                t.base_gap_extra_large,
                t.corner_radius_700,
                t.menu_item_edge_to_content_not_selected_extra_large,
                t.ui_icon_extra_large,
            ),
        };
        let mut label = self.label_type(t);
        label.weight = t.medium_font_weight;
        // The recipe binds the label's line height by relationship at M (the ratio) and by size otherwise.
        label.line_height = match self {
            Size::S => t.line_height_font_size_75,
            Size::M => label.size * t.line_height_100,
            Size::L => t.line_height_font_size_200,
            Size::XL => t.line_height_font_size_300,
        };
        // The value is regular weight and takes the ratio line height at every size (the recipe binds no more).
        let value = Typography {
            weight: t.regular_font_weight,
            line_height: label.size * t.line_height_100,
            ..label
        };
        Dims {
            min_height,
            pad,
            pad_v,
            gap,
            radius,
            not_selected,
            check,
            label,
            value,
        }
    }
}

type Choose = Rc<dyn Fn(El, String) -> El>;
type Dismiss = Rc<dyn Fn(El) -> El>;

pub struct Menu {
    key: String,
    items: Vec<MenuItem>,
    size: Size,
    selection: Selection,
    label: Option<String>,
    choose: Option<Choose>,
    dismiss: Option<Dismiss>,
}

impl Menu {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/menu-item-N` (from 1) and
    /// `{key}/menu-item-N.label`, `.value`, `.checkmark`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        Menu {
            key: key.into(),
            items: Vec::new(),
            size: Size::M,
            selection: Selection::None,
            label: None,
            choose: None,
            dismiss: None,
        }
    }
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn selection(mut self, s: Selection) -> Self {
        self.selection = s;
        self
    }
    /// The menu's accessible name.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Called with the chosen item's id on press, Enter and Space.
    pub fn on_choose<S: 'static>(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        let f = Rc::new(f);
        self.choose = Some(Rc::new(move |e: El, id: String| {
            let (a, b, id2) = (f.clone(), f.clone(), id.clone());
            e.on_press(move |s: &mut S| a(s, &id))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit = matches!(ev.key, Key::Enter) && ev.mods == Mods::default();
                    if hit && !tx.key_repeat() {
                        b(s, &id2);
                    }
                    hit
                })
        }));
        self
    }
    /// Called on Escape while an item has focus.
    pub fn on_dismiss<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        self.dismiss = Some(Rc::new(move |e: El| {
            let f = f.clone();
            e.on_key(move |s: &mut S, ev, _| {
                let hit = matches!(ev.key, Key::Escape) && ev.mods == Mods::default();
                if hit {
                    f(s);
                }
                hit
            })
        }));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        let d = self.size.menu_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let single = self.selection == Selection::Single;
        let n = self.items.len();
        let mut root = col()
            .key(part("container"))
            .gap(t.spacing_50)
            .role("menu")
            .focus_group();
        if let Some(l) = &self.label {
            root = root.label(l.clone());
        }
        for (i, it) in self.items.iter().enumerate() {
            let item_key = part(&format!("menu-item-{}", i + 1));
            let interaction = Interaction::of(&item_key);
            let state = if it.disabled {
                State::Rest
            } else {
                interaction.state()
            };
            let (fill, ink, accent, subdued) = if it.disabled {
                (
                    None,
                    t.disabled_content_color,
                    t.disabled_content_color,
                    t.disabled_content_color,
                )
            } else {
                match state {
                    State::Rest => (
                        None,
                        t.neutral_content_color_default,
                        t.accent_content_color_default,
                        t.neutral_subdued_content_color_default,
                    ),
                    State::Hover => (
                        Some(t.menu_item_background_color_hover),
                        t.neutral_content_color_hover,
                        t.accent_content_color_hover,
                        t.neutral_subdued_content_color_hover,
                    ),
                    State::Down => (
                        Some(t.menu_item_background_color_down),
                        t.neutral_content_color_down,
                        t.accent_content_color_down,
                        t.neutral_subdued_content_color_down,
                    ),
                }
            };
            // The key-focus ink is laid over the content while the item shows keyboard focus.
            let focus = |e: El, c| if it.disabled { e } else { e.group_color(c) };
            let mut line = row()
                .key(item_key.clone())
                .w_full()
                .min_h(d.min_height)
                .py(d.pad_v)
                .pl(d.pad)
                .pr(d.pad)
                .gap(d.gap)
                .center_y()
                .radius(d.radius)
                .role("menuitem")
                .label(it.label.clone())
                .focusable(!it.disabled)
                .disabled(it.disabled)
                .position_in_set(i + 1, n)
                .hover_group_focus();
            if let Some(f) = fill {
                line = line.fill(f);
            }
            if !it.disabled {
                line = line.focus_fill(t.menu_item_background_color_keyboard_focus);
            }
            if single {
                line = line.checked(it.selected);
                if it.selected {
                    line = line.child(focus(
                        icon(&CHECKMARK_100, d.check)
                            .key(format!("{item_key}.checkmark"))
                            .color(accent)
                            .a11y_hidden(),
                        t.accent_content_color_key_focus,
                    ));
                } else {
                    // Line the label up with the selected item's: the checkmark's width, gap and padding.
                    line = line.pl(d.not_selected);
                }
            }
            line = line.child(focus(
                text(it.label.clone())
                    .key(format!("{item_key}.label"))
                    .color(ink)
                    .a11y_hidden()
                    .font(font::text_style(&d.label)),
                t.neutral_content_color_key_focus,
            ));
            if let Some(v) = &it.value {
                line = line.child(el().grow(1.0)).child(focus(
                    text(v.clone())
                        .key(format!("{item_key}.value"))
                        .color(subdued)
                        .a11y_hidden()
                        .font(font::text_style(&d.value)),
                    t.neutral_subdued_content_color_key_focus,
                ));
            }
            if !it.disabled {
                line = interaction.attach(line);
                if let Some(c) = &self.choose {
                    line = c(line, it.id.clone());
                }
                if let Some(x) = &self.dismiss {
                    line = x(line);
                }
            }
            root = root.child(line);
        }
        crate::hug(root)
    }
}
