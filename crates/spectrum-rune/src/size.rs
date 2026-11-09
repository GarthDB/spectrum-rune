//! The component size every Spectrum recipe with the S/M/L/XL `sizeScale` shares (accordion, badge,
//! button, checkbox, radio-button, switch).
//!
//! There is no XS here on purpose: action-button's recipe has one, the others do not. A shared
//! enum with XS would make every other component reject it at run time. Action-button instead
//! defines its own superset (`XS` plus `From<Size>`), so `.size(Size::M)` still works and an XS
//! on a Switch does not compile.

use crate::tokens::{Tokens, Typography};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    S,
    #[default]
    M,
    L,
    XL,
}

impl Size {
    /// Scale-aware label type: `font_size_*` grow on mobile, `component_*_regular` do not.
    pub(crate) fn label_type(self, t: &Tokens) -> Typography {
        let (size, line_height) = match self {
            Size::S => (t.font_size_75, t.line_height_font_size_75),
            Size::M => (t.font_size_100, t.line_height_font_size_100),
            Size::L => (t.font_size_200, t.line_height_font_size_200),
            Size::XL => (t.font_size_300, t.line_height_font_size_300),
        };
        Typography {
            family: t.default_font_family,
            size,
            weight: t.regular_font_weight,
            line_height,
            letter_spacing_em: 0.0,
        }
    }
}
