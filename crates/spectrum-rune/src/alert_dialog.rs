//! Spectrum Alert dialog: a surface with a header (a status icon for the warning and error variants, and the title), a
//! description and a footer holding the dialog's buttons, from `design-data` visuals/alert-dialog.json. Every size, gap,
//! colour and type value comes from [`Tokens`].
//!
//! The dialog is its own element: put it anywhere, or present it with Rune's `El::modal` (as [`crate::popover::Popover`]
//! does). The buttons are the app's own [`crate::button::Button`]s (primary, secondary and cancel; up to three, in ascending
//! importance), passed to [`AlertDialog::actions`] and laid out in a row at the footer's end with `spacing-200` between.
//!
//! Built: the five variants (the status icon for `Warning` and `Error` only, as in the recipe), both scales (the touch scale
//! has the smaller padding and its own title and description sizes), the title and description wrapping, `Escape` calling
//! [`AlertDialog::on_dismiss`]. Limits: the oracle draws no shadow and no scrim, so neither is drawn here (the app's modal
//! layer supplies a scrim if it wants one); the stacked button group of a narrow window is not built.
//!
//! Verification: geometry, type and colours against the oracle (`tests/alert_dialog.rs`); text widths follow the installed
//! font, so only heights, offsets and colours are compared.

use crate::font;
use crate::tokens::{tokens, ColorScheme, Scale, Tokens, Typography};
use rune::{col, icon, row, text, El, Key, Mods};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Confirmation,
    Information,
    Warning,
    Destructive,
    Error,
}

pub struct AlertDialog {
    key: String,
    title: String,
    description: Option<String>,
    variant: Variant,
    actions: Vec<El>,
    dismiss: Option<Rc<dyn Fn(El) -> El>>,
}

/// Whether `t` is the touch scale's token set (its type is larger): the recipe picks different tokens there.
pub(crate) fn touch(t: &Tokens) -> bool {
    t.font_size_100 > tokens(ColorScheme::Light, Scale::Desktop).font_size_100
}

impl AlertDialog {
    /// `key` must be unique per instance: the parts are keyed `{key}/container`, `/header`, `/icon`, `/title`,
    /// `/description`, `/footer` and `/button-group`, as in the recipe.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        AlertDialog {
            key: key.into(),
            title: title.into(),
            description: None,
            variant: Variant::Confirmation,
            actions: Vec::new(),
            dismiss: None,
        }
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
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
        let mobile = touch(t);
        let (padding, title_size, body_size) = if mobile {
            (t.spacing_400, t.font_size_400, t.font_size_100)
        } else {
            (t.spacing_500, t.font_size_500, t.font_size_200)
        };
        let part = |name: &str| format!("{}/{name}", self.key);
        let heading = font::text_style(&Typography {
            size: title_size,
            weight: t.extra_bold_font_weight,
            line_height: title_size * t.heading_line_height,
            ..t.component_m_regular
        });
        let body = font::text_style(&Typography {
            size: body_size,
            weight: t.regular_font_weight,
            line_height: body_size * t.line_height_200,
            ..t.component_m_regular
        });
        let status = match self.variant {
            Variant::Warning => Some(t.notice_visual_color),
            Variant::Error => Some(t.negative_visual_color),
            _ => None,
        };
        let mut header = row().key(part("header")).gap(t.spacing_100).center_y();
        if let Some(c) = status {
            header = header.child(
                icon(
                    &crate::icons::WORKFLOW_ALERT_TRIANGLE,
                    t.workflow_icon_medium,
                )
                .key(part("icon"))
                .color(c)
                .a11y_hidden(),
            );
        }
        header = header.child(
            text(self.title.clone())
                .key(part("title"))
                .font(heading)
                .color(t.heading_color)
                .grow(1.0),
        );
        let mut dialog = col()
            .key(part("container"))
            .w(t.container_maximum_width_2x_large)
            .min_w(t.alert_dialog_minimum_width)
            .padding(padding)
            .gap(t.spacing_300)
            .radius(t.corner_radius_xl)
            .fill(t.background_layer_2_color)
            .role("alertdialog")
            .label(self.title.clone())
            .child(header);
        if let Some(d) = &self.description {
            dialog = dialog.child(
                text(d.clone())
                    .key(part("description"))
                    .font(body)
                    .color(t.body_color),
            );
        }
        if !self.actions.is_empty() {
            dialog = dialog.child(
                row().key(part("footer")).justify_end().child(
                    row()
                        .key(part("button-group"))
                        .gap(t.spacing_200)
                        .children(self.actions),
                ),
            );
        }
        if let Some(d) = &self.dismiss {
            dialog = d(dialog);
        }
        crate::hug(col().child(dialog))
    }
}
