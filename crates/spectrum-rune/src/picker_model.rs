//! The keyboard and selection behaviour of a Picker's list, with no drawing: what a menu on top of
//! [`crate::picker::Picker`] needs, so it can be tested without a window. The field itself only asks to open
//! (`Picker::on_open`); this decides what happens next.
//!
//! The catalog's picker declares the keyboard intents `expand`, `collapse` and `navigate-options` and the
//! role `combobox` (`design-data component picker`). The keys follow the WAI-ARIA Authoring Practices
//! "select-only combobox": Down, Up, Enter and Space open the list; once open, Down and Up move the
//! highlight (no wrap), Home and End jump to the ends, PageUp and PageDown move by ten, Enter and Space
//! choose the highlighted option and close, Tab does the same and lets focus go on, Escape closes without
//! choosing; printable keys highlight the next option starting with what was typed (typing the same letter
//! again cycles). Nothing here knows about Rune: feed it keys and read its state.

/// How long a pause ends a typed prefix.
const TYPEAHEAD_RESET_SECS: f64 = 0.5;
const PAGE: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Space,
    Escape,
    Tab,
    /// A printable character (not Space).
    Char(char),
}

/// What a key did, for the caller to act on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    /// The model used the key: do not let it do anything else (Tab is never swallowed).
    pub handled: bool,
    /// The option chosen by this key, if any.
    pub chosen: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct PickerModel {
    items: Vec<String>,
    open: bool,
    highlighted: Option<usize>,
    selected: Option<usize>,
    typed: String,
    typed_at: f64,
}

impl PickerModel {
    pub fn new(items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        PickerModel {
            items: items.into_iter().map(Into::into).collect(),
            open: false,
            highlighted: None,
            selected: None,
            typed: String::new(),
            typed_at: f64::NEG_INFINITY,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }
    /// The option the keyboard is on while the list is open.
    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
    pub fn selected_item(&self) -> Option<&str> {
        self.selected.map(|i| self.items[i].as_str())
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Chooses an option directly (a click). Closes the list.
    pub fn choose(&mut self, i: usize) {
        if i < self.items.len() {
            self.selected = Some(i);
        }
        self.open = false;
    }

    pub fn open(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.open = true;
        self.highlighted = self.selected.or(Some(0));
    }

    pub fn close(&mut self) {
        self.open = false;
        self.highlighted = None;
        self.typed.clear();
    }

    /// Feeds one key press at time `now` (seconds, any monotonic clock).
    pub fn key(&mut self, key: Key, now: f64) -> Effect {
        if self.items.is_empty() {
            return Effect::default();
        }
        if !self.open {
            return self.key_closed(key, now);
        }
        let last = self.items.len() - 1;
        let at = self.highlighted.unwrap_or(0);
        let go = |m: &mut Self, i: usize| {
            m.highlighted = Some(i.min(last));
            Effect {
                handled: true,
                chosen: None,
            }
        };
        match key {
            Key::Down => go(self, at + 1),
            Key::Up => go(self, at.saturating_sub(1)),
            Key::Home => go(self, 0),
            Key::End => go(self, last),
            Key::PageDown => go(self, at + PAGE),
            Key::PageUp => go(self, at.saturating_sub(PAGE)),
            Key::Enter | Key::Space => self.choose_highlighted(true),
            // Tab chooses what is highlighted and lets focus move on.
            Key::Tab => self.choose_highlighted(false),
            Key::Escape => {
                self.close();
                Effect {
                    handled: true,
                    chosen: None,
                }
            }
            Key::Char(c) => {
                self.type_ahead(c, now);
                Effect {
                    handled: true,
                    chosen: None,
                }
            }
        }
    }

    fn key_closed(&mut self, key: Key, now: f64) -> Effect {
        let last = self.items.len() - 1;
        match key {
            Key::Down | Key::Up | Key::Enter | Key::Space => self.opened(),
            Key::Home => {
                self.opened();
                self.highlighted = Some(0);
                Effect {
                    handled: true,
                    chosen: None,
                }
            }
            Key::End => {
                self.opened();
                self.highlighted = Some(last);
                Effect {
                    handled: true,
                    chosen: None,
                }
            }
            Key::Char(c) => {
                self.opened();
                self.type_ahead(c, now);
                Effect {
                    handled: true,
                    chosen: None,
                }
            }
            Key::PageUp | Key::PageDown | Key::Escape | Key::Tab => Effect::default(),
        }
    }

    fn opened(&mut self) -> Effect {
        self.open();
        Effect {
            handled: true,
            chosen: None,
        }
    }

    fn choose_highlighted(&mut self, handled: bool) -> Effect {
        let chosen = self.highlighted;
        if let Some(i) = chosen {
            self.selected = Some(i);
        }
        self.close();
        Effect { handled, chosen }
    }

    fn type_ahead(&mut self, c: char, now: f64) {
        if now - self.typed_at > TYPEAHEAD_RESET_SECS {
            self.typed.clear();
        }
        self.typed_at = now;
        self.typed.extend(c.to_lowercase());
        let at = self.highlighted.unwrap_or(0);
        let n = self.items.len();
        // The same letter again moves on to the next option with that initial; a longer prefix
        // stays on or after the current option.
        let cycling = self.typed.chars().count() > 1
            && self
                .typed
                .chars()
                .all(|x| x == c.to_lowercase().next().unwrap_or(x));
        let (prefix, skip_current) = if cycling {
            (c.to_lowercase().collect::<String>(), true)
        } else {
            (
                self.typed.clone(),
                self.typed.chars().count() == 1 && self.highlighted.is_some(),
            )
        };
        let start = at + usize::from(skip_current);
        for k in 0..n {
            let i = (start + k) % n;
            if self.items[i].to_lowercase().starts_with(&prefix) {
                self.highlighted = Some(i);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn countries() -> PickerModel {
        PickerModel::new([
            "Argentina",
            "Australia",
            "Austria",
            "Brazil",
            "Canada",
            "Chile",
            "China",
        ])
    }

    #[test]
    fn opening_keys_open_on_the_selection_or_the_first_option() {
        for k in [Key::Down, Key::Up, Key::Enter, Key::Space] {
            let mut m = countries();
            assert!(m.key(k, 0.0).handled);
            assert!(m.is_open(), "{k:?}");
            assert_eq!(m.highlighted(), Some(0));
        }
        let mut m = countries();
        m.choose(4);
        m.key(Key::Down, 0.0);
        assert_eq!(m.highlighted(), Some(4), "opens on the chosen option");
    }

    #[test]
    fn home_and_end_open_on_the_ends() {
        let mut m = countries();
        m.key(Key::End, 0.0);
        assert!(m.is_open());
        assert_eq!(m.highlighted(), Some(6));
        let mut m = countries();
        m.key(Key::Home, 0.0);
        assert_eq!(m.highlighted(), Some(0));
    }

    #[test]
    fn arrows_move_without_wrapping_and_home_end_page_jump() {
        let mut m = countries();
        m.open();
        m.key(Key::Up, 0.0);
        assert_eq!(m.highlighted(), Some(0), "no wrap at the top");
        m.key(Key::Down, 0.0);
        m.key(Key::Down, 0.0);
        assert_eq!(m.highlighted(), Some(2));
        m.key(Key::End, 0.0);
        assert_eq!(m.highlighted(), Some(6));
        m.key(Key::Down, 0.0);
        assert_eq!(m.highlighted(), Some(6), "no wrap at the bottom");
        m.key(Key::Home, 0.0);
        assert_eq!(m.highlighted(), Some(0));
        m.key(Key::PageDown, 0.0);
        assert_eq!(m.highlighted(), Some(6), "a page stops at the end");
        m.key(Key::PageUp, 0.0);
        assert_eq!(m.highlighted(), Some(0));
    }

    #[test]
    fn enter_and_space_choose_and_close() {
        for k in [Key::Enter, Key::Space] {
            let mut m = countries();
            m.open();
            m.key(Key::Down, 0.0);
            let e = m.key(k, 0.0);
            assert_eq!(e.chosen, Some(1));
            assert!(e.handled);
            assert!(!m.is_open());
            assert_eq!(m.selected_item(), Some("Australia"));
        }
    }

    #[test]
    fn escape_closes_without_choosing_and_tab_chooses_but_is_not_swallowed() {
        let mut m = countries();
        m.open();
        m.key(Key::Down, 0.0);
        let e = m.key(Key::Escape, 0.0);
        assert!(e.handled && e.chosen.is_none());
        assert!(!m.is_open());
        assert_eq!(m.selected(), None);
        m.open();
        m.key(Key::Down, 0.0);
        let e = m.key(Key::Tab, 0.0);
        assert_eq!(e.chosen, Some(1));
        assert!(!e.handled, "focus must be allowed to move on");
        assert!(!m.is_open());
        // Closed, Tab and Escape are not ours.
        assert!(!m.key(Key::Tab, 0.0).handled);
        assert!(!m.key(Key::Escape, 0.0).handled);
    }

    #[test]
    fn typing_highlights_the_first_match_and_extends_the_prefix() {
        let mut m = countries();
        m.open();
        m.key(Key::Char('a'), 0.0);
        assert_eq!(
            m.highlighted(),
            Some(1),
            "the first A after the current one"
        );
        m.key(Key::Char('u'), 0.1);
        m.key(Key::Char('s'), 0.2);
        m.key(Key::Char('t'), 0.3);
        m.key(Key::Char('r'), 0.4);
        m.key(Key::Char('i'), 0.5);
        assert_eq!(m.highlighted(), Some(2), "Austri is Austria");
    }

    #[test]
    fn a_pause_starts_a_new_prefix_and_the_same_letter_cycles() {
        let mut m = countries();
        m.open();
        m.key(Key::Char('c'), 0.0);
        assert_eq!(m.highlighted(), Some(4));
        m.key(Key::Char('c'), 0.1);
        assert_eq!(m.highlighted(), Some(5), "C again moves to the next C");
        m.key(Key::Char('c'), 0.2);
        assert_eq!(m.highlighted(), Some(6));
        m.key(Key::Char('c'), 0.3);
        assert_eq!(m.highlighted(), Some(4), "and wraps");
        m.key(Key::Char('b'), 2.0);
        assert_eq!(m.highlighted(), Some(3), "after a pause B starts afresh");
    }

    #[test]
    fn typing_while_closed_opens_on_the_match_and_no_match_stays_put() {
        let mut m = countries();
        m.key(Key::Char('b'), 0.0);
        assert!(m.is_open());
        assert_eq!(m.highlighted(), Some(3));
        m.key(Key::Char('z'), 5.0);
        assert_eq!(m.highlighted(), Some(3), "nothing starts with Z");
    }

    #[test]
    fn a_click_chooses_and_closes_and_an_empty_list_does_nothing() {
        let mut m = countries();
        m.open();
        m.choose(5);
        assert!(!m.is_open());
        assert_eq!(m.selected_item(), Some("Chile"));
        let mut e = PickerModel::new(Vec::<String>::new());
        assert!(!e.key(Key::Down, 0.0).handled);
        e.open();
        assert!(!e.is_open());
        assert!(e.is_empty());
    }

    #[test]
    fn a_long_list_pages_by_ten() {
        let mut m = PickerModel::new((0..97).map(|i| format!("item {i}")));
        m.open();
        m.key(Key::PageDown, 0.0);
        m.key(Key::PageDown, 0.0);
        assert_eq!(m.highlighted(), Some(20));
        m.key(Key::End, 0.0);
        assert_eq!(m.highlighted(), Some(96));
        assert_eq!(m.len(), 97);
    }
}
