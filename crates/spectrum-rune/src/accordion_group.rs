//! Spectrum Accordion: a vertical stack of [`AccordionItem`]s (the catalog's `accordion` with
//! `items` and `isMultiple`). The recipe (`visuals/accordion.json`) has no group part: `items`,
//! `isMultiple` and `down` are `notVisual` and its root is the item, so there is no oracle for the
//! group's layout. It is a column with gap 0, each item keeping its own 1px divider (the last one
//! too), so the stack's height is the sum of the items'; that sum is checked against the item oracle
//! in `tests/accordion_group.rs`.
//!
//! The app owns the open set (`open`), and `on_change` receives the new one, in item order. Single
//! mode (the catalog default, `isMultiple` false) keeps at most one open: pressing the open item
//! closes it, pressing another opens it and closes the rest. Multiple mode toggles membership.
//!
//! Keyboard (WAI-ARIA accordion): every enabled header is a Tab stop, Up and Down move between
//! headers and wrap, Home and End jump to the ends, Space and Enter toggle (the item does that). It
//! does not use Rune's `focus_group`, which leaves one Tab stop, takes any focusable control in an open
//! panel for a header and moves on Left and Right as well. The arrow handler sits on the group and
//! only acts while a header has focus, so a control in a panel keeps its keys. It is attached by
//! `on_change`, which knows the app's state type: a group without one cannot toggle and has no arrows.
//!
//! Per-item `disabled` is a Rune-side convenience: the catalog's `items[]` has no `isDisabled`. A
//! disabled item is not a Tab stop and the arrows skip it.

use crate::accordion::{AccordionItem, Density};
use crate::size::Size;
use crate::tokens::Tokens;
use rune::{col, El, Key, Mods, Tx};
use std::rc::Rc;

/// Gives an item its press, reporting the open set that press would make.
type Toggle = Rc<dyn Fn(AccordionItem, Vec<String>) -> AccordionItem>;
/// Gives the group its arrow keys, over the enabled headers' keys.
type Keys = Rc<dyn Fn(El, Vec<String>) -> El>;

pub struct AccordionGroup {
    key: String,
    label: Option<String>,
    items: Vec<AccordionItem>,
    size: Size,
    density: Density,
    quiet: bool,
    disabled: bool,
    multiple: bool,
    open: Vec<String>,
    wire: Option<(Toggle, Keys)>,
}

impl AccordionGroup {
    /// `key` must be unique per instance: items are keyed `{key}/{item key}`.
    pub fn new(key: impl Into<String>) -> Self {
        AccordionGroup {
            key: key.into(),
            label: None,
            items: Vec::new(),
            size: Size::default(),
            density: Density::default(),
            quiet: false,
            disabled: false,
            multiple: false,
            open: Vec::new(),
            wire: None,
        }
    }
    /// The group's accessible name.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// An item. Its key (unique in the group) is its value in the open set. The group overrides its
    /// size, density, quiet and open, and its toggle; its title, content and `disabled` stay.
    pub fn item(mut self, item: AccordionItem) -> Self {
        debug_assert!(
            self.items.iter().all(|i| i.key != item.key),
            "AccordionGroup `{}`: duplicate item key `{}` (items are keyed by it)",
            self.key,
            item.key
        );
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
    /// No dividers.
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// The catalog's `isMultiple`: several items may be open at once. Off by default.
    pub fn multiple(mut self, v: bool) -> Self {
        self.multiple = v;
        self
    }
    /// The open items' keys. Keys that are not items are ignored. In single mode, give at most one.
    pub fn open<I, V>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        self.open = values.into_iter().map(Into::into).collect();
        self
    }
    /// Called with the new open set (item keys, in item order) on a press, Space or Enter of a
    /// header. Store it as your `open`.
    pub fn on_change<S: 'static>(mut self, f: impl Fn(&mut S, &[String]) + 'static) -> Self {
        let f = Rc::new(f);
        let toggle: Toggle = Rc::new(move |item, next| {
            let f = f.clone();
            item.on_toggle(move |s: &mut S| f(s, &next))
        });
        let keys: Keys = Rc::new(|root, headers| {
            root.on_key(move |_: &mut S, ev, tx: &mut Tx| arrow(&headers, ev, tx))
        });
        self.wire = Some((toggle, keys));
        self
    }

    pub fn build(self, t: &Tokens) -> El {
        debug_assert!(
            self.multiple
                || self
                    .items
                    .iter()
                    .filter(|i| self.open.contains(&i.key))
                    .count()
                    <= 1,
            "AccordionGroup `{}`: more than one item open in single mode",
            self.key
        );
        let values: Vec<String> = self.items.iter().map(|i| i.key.clone()).collect();
        let is_open = |v: &str| self.open.iter().any(|o| o == v);
        let mut headers = Vec::new();
        let mut root = col()
            .key(self.key.clone())
            .role("group")
            .items_stretch()
            .gap(0.0)
            .disabled(self.disabled);
        if let Some(l) = &self.label {
            root = root.label(l.clone());
        }
        for item in self.items {
            let value = item.key.clone();
            let open = is_open(&value);
            let next: Vec<String> = if self.multiple {
                values
                    .iter()
                    .filter(|v| (**v == value) != is_open(v))
                    .cloned()
                    .collect()
            } else if open {
                Vec::new()
            } else {
                vec![value.clone()]
            };
            let mut item = item;
            item.key = format!("{}/{value}", self.key);
            item.disabled |= self.disabled;
            if !item.disabled {
                headers.push(format!("{}/header", item.key));
            }
            let mut item = item
                .size(self.size)
                .density(self.density)
                .quiet(self.quiet)
                .open(open);
            if let Some((toggle, _)) = &self.wire {
                item = toggle(item, next);
            }
            root = root.child(item.build(t));
        }
        match &self.wire {
            Some((_, keys)) => keys(root, headers),
            None => root,
        }
    }
}

/// Up, Down, Home and End between the enabled headers, only while one has focus.
fn arrow(headers: &[String], ev: &rune::KeyEvent, tx: &mut Tx) -> bool {
    if ev.mods != Mods::default() {
        return false;
    }
    let Some(pos) = tx
        .focused()
        .and_then(|f| headers.iter().position(|h| h == f))
    else {
        return false;
    };
    let n = headers.len();
    let next = match ev.key {
        Key::Down => (pos + 1) % n,
        Key::Up => (pos + n - 1) % n,
        Key::Home => 0,
        Key::End => n - 1,
        _ => return false,
    };
    // `Tx::focus` alone leaves a header outside a scroll viewport where it was.
    tx.scroll_into_view(headers[next].clone());
    tx.focus(headers[next].clone(), true);
    true
}
