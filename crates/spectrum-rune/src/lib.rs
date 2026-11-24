#![doc = include_str!("../../../README.md")]

/// `as_str` (the catalog's option value) and `ALL` (every value, in catalog order) for an option enum.
macro_rules! names {
    ($ty:ident { $($v:ident => $s:literal),* $(,)? }) => {
        impl $ty {
            /// The catalog's option value.
            pub fn as_str(self) -> &'static str {
                match self { $($ty::$v => $s),* }
            }
            /// Every value, in catalog order.
            pub const ALL: &'static [$ty] = &[$($ty::$v),*];
        }
    };
}

pub mod accordion;
pub mod accordion_group;
pub mod action_button;
pub mod badge;
mod bar;
pub mod breadcrumbs;
pub mod button;
pub mod checkbox;
pub mod clear_button;
pub mod constraints;
pub mod divider;
pub mod font;
pub mod icons;
pub mod interaction;
pub mod link;
pub mod menu;
pub mod meter;
pub mod picker;
pub mod picker_menu;
pub mod picker_model;
pub mod popover;
pub mod progress_bar;
pub mod progress_circle;
pub mod radio_button;
pub mod radio_group;
pub mod side_navigation;
pub mod size;
pub mod status_light;
pub mod switch;
pub mod tabs;
pub mod tag;
pub mod tokens;
pub mod tooltip;
mod wrap;

pub use size::{ActionButtonSize, Size};

/// Wraps a button-like root so it hugs its content in a column (which stretches its children)
/// and still centres in a row (`align-self: flex-start` on the root itself would stop that).
/// The wrapper stretches or centres; the root, which keeps every key and size, sits at its
/// start.
pub(crate) fn hug(root: rune::El) -> rune::El {
    rune::row().child(root)
}
