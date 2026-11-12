//! Pointer interaction state for Spectrum components.
//!
//! Spectrum swaps whole colours on hover and down, so a component rebuilds with the state
//! this tracks. Keyboard focus needs none: Rune's `focus_ring` shows only for the keyboard.

use rune::{El, Phase};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Rest,
    Hover,
    Down,
}

#[derive(Clone, Default)]
pub struct Interaction(Rc<Inner>);

#[derive(Default)]
struct Inner {
    hover: Cell<bool>,
    down: Cell<bool>,
}

const KEPT_MAX: usize = 1024;

thread_local! {
    static KEPT: RefCell<HashMap<(u64, String), Interaction>> = RefCell::new(HashMap::new());
}

impl Interaction {
    /// The interaction kept for the element `key`, which stays the same across view rebuilds.
    /// Keys must be unique per component instance.
    pub fn of(key: &str) -> Self {
        let id = (rune::current_runtime_id(), key.to_owned());
        KEPT.with_borrow_mut(|kept| {
            // shortcut: clears everything when full (an evicted entry just rests), bound it better if keys churn
            if kept.len() >= KEPT_MAX && !kept.contains_key(&id) {
                kept.clear();
            }
            kept.entry(id).or_default().clone()
        })
    }

    /// Down wins over hover: a held press keeps the pointer inside until it releases.
    pub fn state(&self) -> State {
        match (self.0.down.get(), self.0.hover.get()) {
            (true, _) => State::Down,
            (false, true) => State::Hover,
            _ => State::Rest,
        }
    }

    /// Reports the pointer's hover and press on `e` to this interaction. Skip it for a disabled
    /// component, so it stays at rest. Replaces any hover or drag handler `e` already has.
    pub fn attach(&self, e: El) -> El {
        self.attach_press(self.attach_hover(e))
    }

    /// Only the hover half of [`Interaction::attach`], for a row that wraps the pressable part
    /// and more, so hover holds across all of it.
    pub fn attach_hover(&self, e: El) -> El {
        let hover = self.clone();
        e.on_hover_change_any(move |inside, _| hover.0.hover.set(inside))
    }

    /// Only the press half of [`Interaction::attach`].
    pub fn attach_press(&self, e: El) -> El {
        let down = self.clone();
        e.on_drag_any(move |drag, tx| match drag.phase {
            Phase::Down => down.0.down.set(true),
            Phase::Up => down.0.down.set(false),
            // Moves change nothing the view reads.
            Phase::Move => tx.live_only(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_key_shares_state_and_down_wins() {
        let (a, b, other) = (
            Interaction::of("a"),
            Interaction::of("a"),
            Interaction::of("b"),
        );
        assert_eq!(a.state(), State::Rest);
        a.0.hover.set(true);
        assert_eq!(b.state(), State::Hover);
        a.0.down.set(true);
        assert_eq!(b.state(), State::Down);
        assert_eq!(other.state(), State::Rest);
    }
}
