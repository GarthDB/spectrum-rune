//! Spectrum Standard dialog: a surface with a title, a description and a footer of optional content (such as a checkbox)
//! and the dialog's buttons, from `design-data` visuals/standard-dialog.json. Every size, gap, colour and type value comes
//! from [`Tokens`].
//!
//! Like [`crate::alert_dialog::AlertDialog`] it is its own element (present it with Rune's `El::modal` if it must block the
//! page), and the buttons are the app's [`crate::button::Button`]s passed to [`StandardDialog::actions`].
//!
//! Built: the three sizes (`S`, `M`, `L`: 416, 512 and 608 pt wide), both scales (the touch scale has the smaller padding and
//! its own title and description sizes), the title and description, the footer content, `Escape` calling
//! [`StandardDialog::on_dismiss`]. Limits, mostly the oracle's: the oracle draws the footer without justification, so the
//! buttons sit at the footer's start, or 24 pt after the footer content (not at the end, as Spectrum's design shows); the
//! recipe sets no title weight, so the title is regular (the alert dialog's is extra bold); no shadow or scrim; the image,
//! the close button and the header content slot are not drawn (the recipe has no parts for them).
//!
//! Verification: geometry, type and colours against the oracle (`tests/standard_dialog.rs`); text widths follow the
//! installed font, so only heights, offsets and colours are compared.

use crate::alert_dialog::touch;
use crate::font;
use crate::tokens::{Tokens, Typography};
use rune::{col, row, text, El, Key, Mods};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    S,
    #[default]
    M,
    L,
}

pub struct StandardDialog {
    key: String,
    title: String,
    description: Option<String>,
    size: DialogSize,
    footer_content: Option<El>,
    actions: Vec<El>,
    dismiss: Option<Rc<dyn Fn(El) -> El>>,
}

impl StandardDialog {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `/header`, `/title`, `/body`,
    /// `/description`, `/footer`, `/footer-content` and `/button-group`, as in the recipe.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        StandardDialog {
            key: key.into(),
            title: title.into(),
            description: None,
            size: DialogSize::M,
            footer_content: None,
            actions: Vec::new(),
            dismiss: None,
        }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn size(mut self, s: DialogSize) -> Self {
        self.size = s;
        self
    }
    /// What sits at the start of the footer, such as a [`crate::checkbox::Checkbox`].
    pub fn footer_content(mut self, c: El) -> Self {
        self.footer_content = Some(c);
        self
    }
    /// The dialog's buttons, in ascending importance (the most critical last).
    pub fn actions(mut self, a: impl IntoIterator<Item = El>) -> Self {
        self.actions = a.into_iter().collect();
        self
    }
    /// Called on Escape.
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
        let (padding, title_size, body_size) = if touch(t) {
            (t.spacing_400, t.font_size_400, t.font_size_100)
        } else {
            (t.spacing_500, t.font_size_500, t.font_size_200)
        };
        let width = match self.size {
            DialogSize::S => t.container_maximum_width_extra_large,
            DialogSize::M => t.container_maximum_width_2x_large,
            DialogSize::L => t.container_maximum_width_3x_large,
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let heading = font::text_style(&Typography {
            size: title_size,
            weight: t.regular_font_weight,
            line_height: title_size * t.line_height_100,
            ..t.component_m_regular
        });
        let body = font::text_style(&Typography {
            size: body_size,
            weight: t.regular_font_weight,
            line_height: body_size * t.line_height_200,
            ..t.component_m_regular
        });
        let mut dialog = col()
            .key(part("container"))
            .w(width)
            .min_w(t.standard_dialog_minimum_width)
            .padding(padding)
            .gap(t.spacing_300)
            .radius(t.corner_radius_xl)
            .fill(t.background_layer_2_color)
            .role("dialog")
            .label(self.title.clone())
            .child(
                row().key(part("header")).center_y().child(
                    text(self.title.clone())
                        .key(part("title"))
                        .font(heading)
                        .color(t.heading_color)
                        .grow(1.0),
                ),
            );
        if let Some(d) = &self.description {
            dialog = dialog.child(
                row().key(part("body")).child(
                    text(d.clone())
                        .key(part("description"))
                        .font(body)
                        .color(t.body_color),
                ),
            );
        }
        if self.footer_content.is_some() || !self.actions.is_empty() {
            let mut footer = row().key(part("footer")).gap(t.spacing_400).center_y();
            if let Some(c) = self.footer_content {
                footer = footer.child(row().key(part("footer-content")).center_y().child(c));
            }
            if !self.actions.is_empty() {
                footer = footer.child(
                    row()
                        .key(part("button-group"))
                        .gap(t.spacing_200)
                        .children(self.actions),
                );
            }
            dialog = dialog.child(footer);
        }
        if let Some(d) = &self.dismiss {
            dialog = d(dialog);
        }
        crate::hug(col().child(dialog))
    }
}
