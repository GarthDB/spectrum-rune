//! Spectrum Breadcrumbs: the trail to the current page, from `design-data` visuals/breadcrumbs.json (a row of
//! breadcrumb-items, each a title and a chevron separator except the last). Every size, colour and type value comes
//! from [`Tokens`].
//!
//! Built: items (`label`, `current`, `disabled`), the two sizes (`m`, `l`), the separator (`chevron` or `none`), pressing an
//! item ([`Breadcrumbs::on_navigate`] with its index; the current and disabled ones are inert). Not built: `isMultiline` (wrapping) and
//! `isTruncated` (a collapsed trail with a menu). The recipe has no hover,
//! down or keyboard-focus look (`notVisual`), so none is drawn beyond Rune's own focus ring.
//!
//! Verification: heights, offsets, type and colours against the oracle (`tests/breadcrumbs.rs`); text widths follow the
//! installed font, so the oracle's widths are not compared.

use crate::font;
use crate::icons::{CHEVRON_100, CHEVRON_200};
use crate::tokens::{Tokens, Typography};
use rune::{icon, row, text, El, Key, Mods};
use std::rc::Rc;

/// The sizes the recipe has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    #[default]
    M,
    L,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Separator {
    #[default]
    Chevron,
    None,
}

#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    label: String,
    current: bool,
    disabled: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        BreadcrumbItem {
            label: label.into(),
            current: false,
            disabled: false,
        }
    }
    /// The page the trail is on: bold, and not a link.
    pub fn current(mut self, v: bool) -> Self {
        self.current = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
}

type Navigate = Rc<dyn Fn(El, usize) -> El>;

pub struct Breadcrumbs {
    key: String,
    items: Vec<BreadcrumbItem>,
    size: Size,
    separator: Separator,
    label: String,
    navigate: Option<Navigate>,
}

impl Breadcrumbs {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `{key}/breadcrumb-item-N` (from 1) and
    /// `{key}/breadcrumb-item-N.breadcrumb-title`, `.separator`, as in the recipe.
    pub fn new(key: impl Into<String>) -> Self {
        Breadcrumbs {
            key: key.into(),
            items: Vec::new(),
            size: Size::M,
            separator: Separator::Chevron,
            label: "Breadcrumbs".to_owned(),
            navigate: None,
        }
    }
    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn separator(mut self, s: Separator) -> Self {
        self.separator = s;
        self
    }
    /// The navigation's accessible name (`Breadcrumbs` by default).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Called with the item's index on press and on Enter. The current and disabled items do not call it.
    pub fn on_navigate<S: 'static>(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        let f = Rc::new(f);
        self.navigate = Some(Rc::new(move |e: El, i: usize| {
            let (a, b) = (f.clone(), f.clone());
            e.on_press(move |s: &mut S| a(s, i))
                .on_key(move |s: &mut S, ev, tx| {
                    // A link follows on Enter, once per press.
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
        let (height, gap, font_size, chevron) = match self.size {
            Size::M => (
                t.base_height_medium,
                t.base_gap_medium,
                t.font_size_100,
                &CHEVRON_100,
            ),
            Size::L => (
                t.base_height_large,
                t.base_gap_2x_large,
                t.font_size_200,
                &CHEVRON_200,
            ),
        };
        let n = self.items.len();
        let part = |name: &str| format!("{}/{name}", self.key);
        let mut trail = row()
            .key(part("container"))
            .center_y()
            .min_h(height)
            .role("navigation")
            .label(self.label.clone());
        for (i, it) in self.items.iter().enumerate() {
            let item_key = part(&format!("breadcrumb-item-{}", i + 1));
            let (weight, ink) = if it.disabled {
                (t.regular_font_weight, t.disabled_content_color)
            } else if it.current {
                (t.bold_font_weight, t.neutral_content_color_default)
            } else {
                (
                    t.regular_font_weight,
                    t.neutral_subdued_content_color_default,
                )
            };
            let style = Typography {
                size: font_size,
                weight,
                line_height: font_size * t.line_height_100,
                ..t.component_m_regular
            };
            let mut line = row()
                .key(item_key.clone())
                .center_y()
                .gap(gap)
                .pl(gap)
                .role(if it.current || it.disabled {
                    "text"
                } else {
                    "link"
                })
                .label(it.label.clone())
                .disabled(it.disabled)
                .child(
                    text(it.label.clone())
                        .key(format!("{item_key}.breadcrumb-title"))
                        .font(font::text_style(&style))
                        .color(ink)
                        .a11y_hidden(),
                );
            if self.separator == Separator::Chevron && i + 1 < n {
                line = line.child(
                    icon(chevron, 16.0)
                        .key(format!("{item_key}.separator"))
                        .color(t.neutral_subdued_content_color_default)
                        .a11y_hidden(),
                );
            }
            if !it.current && !it.disabled {
                line = line.focusable(true);
                if let Some(nav) = &self.navigate {
                    line = nav(line, i);
                }
            }
            trail = trail.child(line);
        }
        crate::hug(trail)
    }
}
