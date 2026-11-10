#![doc = include_str!("../../../README.md")]

pub mod accordion;
pub mod action_button;
pub mod badge;
pub mod button;
pub mod checkbox;
pub mod constraints;
pub mod font;
pub mod icons;
pub mod interaction;
pub mod radio_button;
pub mod size;
pub mod switch;
pub mod tokens;
mod wrap;

pub use size::{ActionButtonSize, Size};

/// Wraps a button-like root so it hugs its content in a column (which stretches its children)
/// and still centres in a row (`align-self: flex-start` on the root itself would stop that).
/// The wrapper stretches or centres; the root, which keeps every key and size, sits at its
/// start.
pub(crate) fn hug(root: rune::El) -> rune::El {
    rune::row().child(root)
}
