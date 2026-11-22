//! Spectrum Picker, the closed field only (the menu and popover are not built), from `design-data`
//! visuals/picker.json: container > field-label, field (value | placeholder, disclosure-icon), focus
//! ring. Every size, colour, radius and type value comes from [`Tokens`], except the disclosure
//! icon's 16 px, which neither the recipe nor the tokens name (the oracle's renderer default).
//!
//! What the recipe does not draw, and so this does not either: `labelPosition: side` (the label
//! stays on top), `isQuiet` beyond the smaller end padding (the fill stays the field fill), a
//! disabled fill (only the ink dims), a hover or key-focus ink for the placeholder, and a focus
//! look beyond the ring and the key-focus ink, `necessityIndicator: text` and `errorMessage` (nothing is drawn for
//! them). `description` is the help text under the field (the alert icon and negative colour when `error`), and
//! `required` the asterisk after the label.
//!
//! Press, Space, Enter and Arrow Down call [`Picker::on_open`]; the app opens its own menu and
//! tells the field through [`Picker::open`] (accessibility `expanded`).
//!
//! Verification: geometry and colours against the oracle (`tests/picker.rs`); the focus ring from
//! pixels (`tests/picker_pixels.rs`). The font family is not asserted (not observable).

use crate::button::press_scale;
use crate::constraints::{self, OptionValue};
use crate::font;
use crate::icons::{
    ASTERISK_100, ASTERISK_200, ASTERISK_300, CHEVRON_100, WORKFLOW_ALERT_TRIANGLE,
};
use crate::interaction::{Interaction, State};
use crate::tokens::{Tokens, Typography};
use rune::tok::motion::SNAPPY;
use rune::{col, el, icon, row, text, El, Icon, Key, Mods};
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

pub use crate::size::Size;

/// The disclosure icon's drawn size: not in the recipe or the tokens (the oracle draws 16 at every size).
const DISCLOSURE: f32 = 16.0;

/// The per-size tokens the recipe reads.
struct Dims {
    height: f32,
    width: f32,
    pad: f32,
    pad_end_quiet: f32,
    radius: f32,
    value: Typography,
    label: Typography,
    gap: f32,
    asterisk: (&'static Icon, f32),
    alert: f32,
}

impl Size {
    fn picker_dims(self, t: &Tokens) -> Dims {
        let (height, width, pad, radius, value) = match self {
            Size::S => (
                t.base_height_small,
                t.field_width_small,
                t.base_padding_horizontal_small,
                t.corner_radius_medium_size_small,
                t.component_s_regular,
            ),
            Size::M => (
                t.base_height_medium,
                t.field_width_medium,
                t.base_padding_horizontal_medium,
                t.corner_radius_medium_size_medium,
                t.component_m_regular,
            ),
            Size::L => (
                t.base_height_large,
                t.field_width_large,
                t.base_padding_horizontal_large,
                t.corner_radius_medium_size_large,
                t.component_l_regular,
            ),
            Size::XL => (
                t.base_height_extra_large,
                t.field_width_extra_large,
                t.base_padding_horizontal_extra_large,
                t.corner_radius_medium_size_extra_large,
                t.component_xl_regular,
            ),
        };
        let (gap, asterisk, alert) = match self {
            Size::S => (
                t.base_gap_small,
                (&ASTERISK_100, t.asterisk_size_100),
                t.workflow_icon_small,
            ),
            Size::M => (
                t.base_gap_medium,
                (&ASTERISK_100, t.asterisk_size_100),
                t.workflow_icon_medium,
            ),
            Size::L => (
                t.base_gap_large,
                (&ASTERISK_200, t.asterisk_size_200),
                t.workflow_icon_large,
            ),
            Size::XL => (
                t.base_gap_extra_large,
                (&ASTERISK_300, t.asterisk_size_300),
                t.workflow_icon_extra_large,
            ),
        };
        Dims {
            height,
            width,
            pad,
            pad_end_quiet: t.picker_end_edge_to_disclosure_icon_quiet,
            radius,
            value,
            // The label's line height is the unitless `line-height-100` times its size.
            label: Typography {
                line_height: t.line_height_100 * value.size,
                ..value
            },
            gap,
            asterisk,
            alert,
        }
    }
}

type Open = Box<dyn FnOnce(El) -> El>;

pub struct Picker {
    key: String,
    label: Option<String>,
    placeholder: Option<String>,
    value: Option<String>,
    size: Size,
    quiet: bool,
    disabled: bool,
    error: bool,
    required: bool,
    description: Option<String>,
    open: bool,
    width: Option<f32>,
    on_open: Option<Open>,
}

impl Picker {
    /// `key` must be unique per instance: it keys the pointer state, and the parts are keyed
    /// `{key}/{recipe part}` (`container`, `field`, `value`, `placeholder`, `disclosure-icon`,
    /// `field-label`, `field-label.label`), so `rt.find("{key}/field")` works.
    pub fn new(key: impl Into<String>) -> Self {
        Picker {
            key: key.into(),
            label: None,
            placeholder: None,
            value: None,
            size: Size::M,
            quiet: false,
            disabled: false,
            error: false,
            required: false,
            description: None,
            open: false,
            width: None,
            on_open: None,
        }
    }
    /// Shown above the field and read as the field's accessible name. Every picker should have one.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Shown while there is no `value`.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    /// The chosen item's text. It wins over the placeholder.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn quiet(mut self, v: bool) -> Self {
        self.quiet = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }
    /// Whether the app's menu is open: only the accessibility `expanded` state follows it.
    pub fn open(mut self, v: bool) -> Self {
        self.open = v;
        self
    }
    /// Field width in points (the oracle's `--width`); the size's `field-width` token without it.
    /// Shows the required asterisk after the label (`isRequired` with the icon necessity indicator; the oracle does
    /// not draw the text indicator).
    pub fn required(mut self, v: bool) -> Self {
        self.required = v;
        self
    }
    /// Help text under the field. With [`Picker::error`] it is the error text: negative colour and the alert icon.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// Called on press, Space, Enter and Arrow Down; open your own menu here.
    pub fn on_open<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        let f = Rc::new(f);
        // Rune presses with Space only (Return is the window's default button); a combobox also
        // opens on Enter and Arrow Down, as on the web.
        self.on_open = Some(Box::new(move |e| {
            let key = f.clone();
            e.on_press(move |s: &mut S| f(s))
                .on_key(move |s: &mut S, ev, tx| {
                    let hit =
                        matches!(ev.key, Key::Enter | Key::Down) && ev.mods == Mods::default();
                    // Once per press, as Space does: a held key must not reopen again and again.
                    if hit && !tx.key_repeat() {
                        key(s);
                    }
                    hit
                })
        }));
        self
    }

    /// The builder state under the catalog's option names.
    pub fn options(&self) -> Vec<(&'static str, OptionValue<'static>)> {
        let mut o = vec![
            ("size", OptionValue::Str(self.size.as_str())),
            ("isQuiet", OptionValue::Bool(self.quiet)),
            ("isDisabled", OptionValue::Bool(self.disabled)),
            ("isError", OptionValue::Bool(self.error)),
            ("isRequired", OptionValue::Bool(self.required)),
        ];
        if self.description.is_some() {
            o.push(("description", OptionValue::Present));
        }
        o
    }

    pub fn build(self, t: &Tokens) -> El {
        constraints::assert_valid("picker", &self.options());
        let d = self.size.picker_dims(t);
        let part = |name: &str| format!("{}/{name}", self.key);
        let interaction = Interaction::of(&self.key);
        let state = if self.disabled {
            State::Rest
        } else {
            interaction.state()
        };
        let (fill, ink) = match state {
            State::Rest => (t.gray_100, t.neutral_content_color_default),
            State::Hover => (t.gray_200, t.neutral_content_color_hover),
            State::Down => (t.gray_100, t.neutral_content_color_down),
        };
        let (ink, subdued) = if self.disabled {
            (t.disabled_content_color, t.disabled_content_color)
        } else {
            (ink, t.neutral_subdued_content_color_default)
        };
        let width = self.width.unwrap_or(d.width);
        let pad_end = if self.quiet { d.pad_end_quiet } else { d.pad };

        let mut field = row()
            .key(part("field"))
            .w(width)
            .h(d.height)
            .center_y()
            .pl(d.pad)
            .pr(pad_end)
            .radius(d.radius)
            .fill(fill)
            .token_motion(SNAPPY)
            .role("combobox")
            .expanded(self.open)
            .focusable(!self.disabled)
            .disabled(self.disabled)
            .focus_ring(
                t.focus_indicator_thickness,
                t.focus_ring_gap,
                t.focus_indicator_color,
            )
            .hover_group_focus();
        if self.error {
            field = field.border(t.border_width_100, t.negative_border_color_default);
        }
        if let Some(l) = &self.label {
            field = field.label(l.clone());
        }
        if let Some(v) = &self.value {
            field = field.a11y_value(v.clone());
        }
        // The help text is the field's description for assistive technology (the text node itself is hidden).
        if let Some(h) = &self.description {
            field = field.description(h.clone());
        }

        // The key-focus ink is laid over the value and the icon while the field shows keyboard focus.
        let focus_ink = |e: El| {
            if self.disabled {
                e
            } else {
                e.group_color(t.neutral_content_color_key_focus)
            }
        };
        // A value wins over the placeholder; neither draws no text, as in the oracle.
        let line = |name: &str, s: String, color| {
            text(s)
                .key(part(name))
                .color(color)
                .a11y_hidden()
                .font(font::text_style(&d.value))
                .shrink()
                .truncate()
        };
        field = match (self.value, self.placeholder) {
            (Some(v), _) => field.child(focus_ink(line("value", v, ink))),
            (None, Some(p)) => field.child(line("placeholder", p, subdued)),
            _ => field,
        };
        // The text keeps its own width (the oracle's rect); the spacer takes the rest, so the icon sits at the end.
        field = field.child(el().grow(1.0)).child(focus_ink(
            icon(&CHEVRON_100, DISCLOSURE)
                .key(part("disclosure-icon"))
                .color(ink)
                .icon_rotation(FRAC_PI_2)
                .a11y_hidden(),
        ));

        if !self.disabled {
            field = interaction.attach(field).press_scale(press_scale(t, width));
            if let Some(open) = self.on_open {
                field = open(field);
            }
        }

        let mut root = col().key(part("container")).items_start().w(width);
        if let Some(l) = self.label {
            let label = text(l)
                .key(part("field-label.label"))
                .color(subdued)
                .a11y_hidden()
                .font(font::text_style(&d.label));
            // The asterisk sits after the text, top-aligned with it.
            let mut line = row().gap(d.gap).items_start().child(label);
            if self.required {
                line = line.child(
                    icon(d.asterisk.0, d.asterisk.1)
                        .key(part("field-label.necessity-indicator"))
                        .color(subdued)
                        .a11y_hidden(),
                );
            }
            root = root.child(
                row()
                    .key(part("field-label"))
                    .h(d.height)
                    .center_y()
                    .child(line),
            );
        }
        root = root.child(field);
        if let Some(help) = self.description {
            let (color, alert) = if self.error {
                (t.negative_content_color_default, true)
            } else {
                (subdued, false)
            };
            // The row is one text line high; on a mobile scale the icon is taller and overflows it, as in the oracle.
            let mut line = row()
                .key(part("help-text"))
                .h(d.label.line_height)
                .gap(d.gap)
                .items_start();
            if alert {
                line = line.child(
                    icon(&WORKFLOW_ALERT_TRIANGLE, d.alert)
                        .key(part("help-text.icon"))
                        .color(color)
                        .a11y_hidden(),
                );
            }
            root = root.child(
                line.child(
                    text(help)
                        .key(part("help-text.text"))
                        .color(color)
                        .font(font::text_style(&d.label)),
                ),
            );
        }
        crate::hug(root)
    }
}
