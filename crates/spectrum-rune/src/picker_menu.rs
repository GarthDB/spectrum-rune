//! A Picker's menu: the [`Menu`] in a [`Popover`] anchored below the picker's field, as Spectrum's picker opens it
//! (no tip, lined up with the field's start, as wide as the field). It is modal: focus goes to the selected item (or
//! the first), Tab stays inside, Escape or a press outside calls [`PickerMenu::on_close`], and focus returns to the
//! field. The Picker itself only asks to open (`Picker::on_open`); the app shows this while its menu is open, mounted
//! at the root of the view after the picker so it draws above later siblings and escapes clipping ancestors:
//!
//! ```ignore
//! if s.open {
//!     root = root.child(
//!         PickerMenu::new("country")        // the Picker's key
//!             .item("ca", "Canada")
//!             .item("mx", "Mexico")
//!             .selected("ca")
//!             .on_choose(|s: &mut App, id: &str| { s.country = id.into(); s.open = false })
//!             .on_close(|s: &mut App| s.open = false)
//!             .build(t),
//!     );
//! }
//! ```
//!
//! Built from the catalog's picker (`menuContainer: popover`); the tray form for small screens is not built.

use crate::menu::{Menu, MenuItem, Selection};
use crate::popover::{Placement, Popover};
use crate::tokens::Tokens;
use rune::{Align, El};

pub use crate::size::Size;

pub struct PickerMenu {
    key: String,
    menu: Menu,
    popover: Popover,
    size: Size,
    selected: Option<String>,
    items: Vec<(String, String)>,
    width: Option<f32>,
}

impl PickerMenu {
    /// `key` is the Picker's key: the menu anchors to `{key}/field` and is keyed `{key}/menu`.
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        PickerMenu {
            menu: Menu::new(format!("{key}/menu"))
                .selection(Selection::Single)
                .autofocus(true),
            popover: Popover::new(format!("{key}/popover"))
                .placement(Placement::Bottom)
                .hide_tip(true)
                .align(Align::Start)
                .offset(4.0)
                .anchor(format!("{key}/field")),
            key,
            size: Size::M,
            selected: None,
            items: Vec::new(),
            width: None,
        }
    }
    pub fn item(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.items.push((id.into(), label.into()));
        self
    }
    pub fn selected(mut self, id: impl Into<String>) -> Self {
        self.selected = Some(id.into());
        self
    }
    /// The picker's size: the items are drawn at it.
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    /// The popover's width, normally the field's.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.menu = self.menu.label(l);
        self
    }
    /// Called with the chosen item's id on press, Enter and Space. Closing is the app's to do.
    pub fn on_choose<S: 'static>(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.menu = self.menu.on_choose(f);
        self
    }
    /// Called on Escape and on a press outside.
    pub fn on_close<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let field = format!("{}/field", self.key);
        self.popover = self.popover.modal(field, f);
        self
    }
    pub fn build(self, t: &Tokens) -> El {
        let mut menu = self.menu.size(self.size);
        for (id, label) in &self.items {
            let sel = self.selected.as_deref() == Some(id.as_str());
            menu = menu.item(MenuItem::new(id.clone(), label.clone()).selected(sel));
        }
        let mut pop = self.popover.child(menu.build(t));
        if let Some(w) = self.width {
            pop = pop.width(w);
        }
        pop.build(t)
    }
}
